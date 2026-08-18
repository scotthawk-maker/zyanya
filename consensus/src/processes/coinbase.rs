use zyanya_consensus_core::{
    coinbase::*,
    errors::coinbase::{CoinbaseError, CoinbaseResult},
    subnets,
    tx::{ScriptPublicKey, ScriptVec, Transaction, TransactionOutput},
    BlockHashMap, BlockHashSet,
};
use std::convert::TryInto;
use zyanya_txscript::{opcodes::codes::OpCheckSequenceVerify, script_builder::ScriptBuilder};

use crate::{constants, model::stores::ghostdag::GhostdagData};

const LENGTH_OF_BLUE_SCORE: usize = size_of::<u64>();
const LENGTH_OF_SUBSIDY: usize = size_of::<u64>();
const LENGTH_OF_SCRIPT_PUB_KEY_VERSION: usize = size_of::<u16>();
const LENGTH_OF_SCRIPT_PUB_KEY_LENGTH: usize = size_of::<u8>();

const MIN_PAYLOAD_LENGTH: usize =
    LENGTH_OF_BLUE_SCORE + LENGTH_OF_SUBSIDY + LENGTH_OF_SCRIPT_PUB_KEY_VERSION + LENGTH_OF_SCRIPT_PUB_KEY_LENGTH;

// We define a year as 365.25 days and a month as 365.25 / 12 = 30.4375
// SECONDS_PER_MONTH = 30.4375 * 24 * 60 * 60
const SECONDS_PER_MONTH: u64 = 2629800;

pub const SUBSIDY_BY_MONTH_TABLE_SIZE: usize = 727;
pub type SubsidyByMonthTable = [u64; SUBSIDY_BY_MONTH_TABLE_SIZE];

#[derive(Clone)]
pub struct CoinbaseManager {
    coinbase_payload_script_public_key_max_len: u8,
    max_coinbase_payload_len: usize,
    deflationary_phase_daa_score: u64,
    pre_deflationary_phase_base_subsidy: u64,
    target_time_per_block: u64,

    /// Precomputed number of blocks per month
    blocks_per_month: u64,

    /// Precomputed subsidy by month table
    subsidy_by_month_table: SubsidyByMonthTable,
}

/// Struct used to streamline payload parsing
struct PayloadParser<'a> {
    remaining: &'a [u8], // The unparsed remainder
}

impl<'a> PayloadParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Returns a slice with the first `n` bytes of `remaining`, while setting `remaining` to the remaining part
    fn take(&mut self, n: usize) -> &[u8] {
        let (segment, remaining) = self.remaining.split_at(n);
        self.remaining = remaining;
        segment
    }
}

/// Helper function to attach a CSV relative timelock to a ScriptPublicKey.
/// Prepends `<lock_blocks> OP_CHECKSEQUENCEVERIFY` to `base_spk`.
pub fn create_csv_locked_script(base_spk: &ScriptPublicKey, lock_blocks: u64) -> ScriptPublicKey {
    let mut builder = ScriptBuilder::new();
    let _ = builder.add_sequence(lock_blocks);
    let _ = builder.add_op(OpCheckSequenceVerify);
    let mut script = builder.drain();
    script.extend_from_slice(base_spk.script());
    ScriptPublicKey::new(base_spk.version(), ScriptVec::from_slice(&script))
}

impl CoinbaseManager {
    pub fn new(
        coinbase_payload_script_public_key_max_len: u8,
        max_coinbase_payload_len: usize,
        deflationary_phase_daa_score: u64,
        pre_deflationary_phase_base_subsidy: u64,
        target_time_per_block: u64,
    ) -> Self {
        assert!(1000 % target_time_per_block == 0);
        let bps = 1000 / target_time_per_block;
        let blocks_per_month = SECONDS_PER_MONTH * bps;

        // Precomputed subsidy by month table for the actual block per second rate.
        // If deflationary_phase_daa_score == 31_449_600 (Mainnet), we use the precomputed
        // MAINNET_SUBSIDY_BY_MONTH_TABLE — a deterministic integer schedule for the
        // smooth geometric decay calibrated for ~28.7B ZYAN cap: initial reward = 50
        // ZYAN/block (5,000,000,000 sompi), monthly decay rate = 0.004847033515
        // (0.4847033515%). This table was computed with exact rational arithmetic (no
        // floating point) to guarantee bit-identical results across all platforms and
        // compilers. The previous f64::powf computation was non-deterministic and could
        // cause a consensus fork.
        let subsidy_by_month_table: SubsidyByMonthTable = if deflationary_phase_daa_score == 31_449_600 {
            core::array::from_fn(|i| MAINNET_SUBSIDY_BY_MONTH_TABLE[i].div_ceil(bps))
        } else {
            core::array::from_fn(|i| SUBSIDY_BY_MONTH_TABLE[i].div_ceil(bps))
        };

        Self {
            coinbase_payload_script_public_key_max_len,
            max_coinbase_payload_len,
            deflationary_phase_daa_score,
            pre_deflationary_phase_base_subsidy,
            target_time_per_block,
            blocks_per_month,
            subsidy_by_month_table,
        }
    }

    #[cfg(test)]
    #[inline]
    pub fn bps(&self) -> u64 {
        1000 / self.target_time_per_block
    }

    pub fn expected_coinbase_transaction<T: AsRef<[u8]>>(
        &self,
        daa_score: u64,
        miner_data: MinerData<T>,
        ghostdag_data: &GhostdagData,
        mergeset_rewards: &BlockHashMap<BlockRewardData>,
        mergeset_non_daa: &BlockHashSet,
    ) -> CoinbaseResult<CoinbaseTransactionTemplate> {
        let mut outputs = Vec::with_capacity((ghostdag_data.mergeset_blues.len() + 1) * 13); // + 1 for possible red reward

        // Add outputs for each mergeset blue block (∩ DAA window), paying to the script reported by the block.
        // Note that combinatorically it is nearly impossible for a blue block to be non-DAA
        for blue in ghostdag_data.mergeset_blues.iter().filter(|h| !mergeset_non_daa.contains(h)) {
            let reward_data = mergeset_rewards.get(blue).unwrap();
            let total_reward = reward_data.subsidy + reward_data.total_fees;
            if total_reward > 0 {
                let liquid_amount = total_reward / 2;
                let vested_amount = total_reward - liquid_amount;

                // 50% Liquid output (immediate, spendable after coinbase maturity 100 blocks)
                if liquid_amount > 0 {
                    outputs.push(TransactionOutput::new(liquid_amount, reward_data.script_public_key.clone()));
                }

                // 50% Vested outputs (released linearly over 12 monthly periods with CSV time-locks)
                if vested_amount > 0 {
                    let monthly_amount = vested_amount / 12;
                    let remainder = vested_amount - (monthly_amount * 11);
                    for i in 0..12 {
                        let amount = if i == 11 { remainder } else { monthly_amount };
                        if amount > 0 {
                            let lock_blocks = (i as u64 + 1) * self.blocks_per_month;
                            let locked_spk = create_csv_locked_script(&reward_data.script_public_key, lock_blocks);
                            outputs.push(TransactionOutput::new(amount, locked_spk));
                        }
                    }
                }
            }
        }

        // Collect all rewards from mergeset reds ∩ DAA window and create
        // outputs rewarding all to the current block (the "merging" block)
        let mut red_reward = 0u64;
        for red in ghostdag_data.mergeset_reds.iter().filter(|h| !mergeset_non_daa.contains(h)) {
            let reward_data = mergeset_rewards.get(red).unwrap();
            red_reward += reward_data.subsidy + reward_data.total_fees;
        }
        if red_reward > 0 {
            let liquid_amount = red_reward / 2;
            let vested_amount = red_reward - liquid_amount;

            // 50% Liquid output
            if liquid_amount > 0 {
                outputs.push(TransactionOutput::new(liquid_amount, miner_data.script_public_key.clone()));
            }

            // 50% Vested outputs (12 monthly linear outputs with CSV time-locks)
            if vested_amount > 0 {
                let monthly_amount = vested_amount / 12;
                let remainder = vested_amount - (monthly_amount * 11);
                for i in 0..12 {
                    let amount = if i == 11 { remainder } else { monthly_amount };
                    if amount > 0 {
                        let lock_blocks = (i as u64 + 1) * self.blocks_per_month;
                        let locked_spk = create_csv_locked_script(&miner_data.script_public_key, lock_blocks);
                        outputs.push(TransactionOutput::new(amount, locked_spk));
                    }
                }
            }
        }

        // Build the current block's payload
        let subsidy = self.calc_block_subsidy(daa_score);
        let payload = self.serialize_coinbase_payload(&CoinbaseData { blue_score: ghostdag_data.blue_score, subsidy, miner_data })?;

        Ok(CoinbaseTransactionTemplate {
            tx: Transaction::new(constants::TX_VERSION, vec![], outputs, 0, subnets::SUBNETWORK_ID_COINBASE, 0, payload),
            has_red_reward: red_reward > 0,
        })
    }

    pub fn serialize_coinbase_payload<T: AsRef<[u8]>>(&self, data: &CoinbaseData<T>) -> CoinbaseResult<Vec<u8>> {
        let script_pub_key_len = data.miner_data.script_public_key.script().len();
        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len as usize {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }
        let payload: Vec<u8> = data.blue_score.to_le_bytes().iter().copied()                    // Blue score                   (u64)
            .chain(data.subsidy.to_le_bytes().iter().copied())                                  // Subsidy                      (u64)
            .chain(data.miner_data.script_public_key.version().to_le_bytes().iter().copied())   // Script public key version    (u16)
            .chain((script_pub_key_len as u8).to_le_bytes().iter().copied())                    // Script public key length     (u8)
            .chain(data.miner_data.script_public_key.script().iter().copied())                  // Script public key            
            .chain(data.miner_data.extra_data.as_ref().iter().copied())                         // Extra data
            .collect();

        Ok(payload)
    }

    pub fn modify_coinbase_payload<T: AsRef<[u8]>>(&self, mut payload: Vec<u8>, miner_data: &MinerData<T>) -> CoinbaseResult<Vec<u8>> {
        let script_pub_key_len = miner_data.script_public_key.script().len();
        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len as usize {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }

        // Keep only blue score and subsidy. Note that truncate does not modify capacity, so
        // the usual case where the payloads are the same size will not trigger a reallocation
        payload.truncate(LENGTH_OF_BLUE_SCORE + LENGTH_OF_SUBSIDY);
        payload.extend(
            miner_data.script_public_key.version().to_le_bytes().iter().copied() // Script public key version (u16)
                .chain((script_pub_key_len as u8).to_le_bytes().iter().copied()) // Script public key length  (u8)
                .chain(miner_data.script_public_key.script().iter().copied())    // Script public key
                .chain(miner_data.extra_data.as_ref().iter().copied()), // Extra data
        );

        Ok(payload)
    }

    pub fn deserialize_coinbase_payload<'a>(&self, payload: &'a [u8]) -> CoinbaseResult<CoinbaseData<&'a [u8]>> {
        if payload.len() < MIN_PAYLOAD_LENGTH {
            return Err(CoinbaseError::PayloadLenBelowMin(payload.len(), MIN_PAYLOAD_LENGTH));
        }

        if payload.len() > self.max_coinbase_payload_len {
            return Err(CoinbaseError::PayloadLenAboveMax(payload.len(), self.max_coinbase_payload_len));
        }

        let mut parser = PayloadParser::new(payload);

        let blue_score = u64::from_le_bytes(parser.take(LENGTH_OF_BLUE_SCORE).try_into().unwrap());
        let subsidy = u64::from_le_bytes(parser.take(LENGTH_OF_SUBSIDY).try_into().unwrap());
        let script_pub_key_version = u16::from_le_bytes(parser.take(LENGTH_OF_SCRIPT_PUB_KEY_VERSION).try_into().unwrap());
        let script_pub_key_len = u8::from_le_bytes(parser.take(LENGTH_OF_SCRIPT_PUB_KEY_LENGTH).try_into().unwrap());

        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len as usize,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }

        if parser.remaining.len() < script_pub_key_len as usize {
            return Err(CoinbaseError::PayloadCantContainScriptPublicKey(
                payload.len(),
                MIN_PAYLOAD_LENGTH + script_pub_key_len as usize,
            ));
        }

        let script_public_key =
            ScriptPublicKey::new(script_pub_key_version, ScriptVec::from_slice(parser.take(script_pub_key_len as usize)));
        let extra_data = parser.remaining;

        Ok(CoinbaseData { blue_score, subsidy, miner_data: MinerData { script_public_key, extra_data } })
    }

    pub fn calc_block_subsidy(&self, daa_score: u64) -> u64 {
        if daa_score < self.deflationary_phase_daa_score {
            return self.pre_deflationary_phase_base_subsidy;
        }

        let months_since_deflationary_phase_started =
            ((daa_score - self.deflationary_phase_daa_score) / self.blocks_per_month) as usize;
        if months_since_deflationary_phase_started >= self.subsidy_by_month_table.len() {
            *(self.subsidy_by_month_table).last().unwrap()
        } else {
            self.subsidy_by_month_table[months_since_deflationary_phase_started]
        }
    }

    #[cfg(test)]
    pub fn legacy_calc_block_subsidy(&self, daa_score: u64) -> u64 {
        if daa_score < self.deflationary_phase_daa_score {
            return self.pre_deflationary_phase_base_subsidy;
        }

        // Note that this calculation implicitly assumes that block per second = 1 (by assuming daa score diff is in second units).
        let months_since_deflationary_phase_started = (daa_score - self.deflationary_phase_daa_score) / SECONDS_PER_MONTH;
        assert!(months_since_deflationary_phase_started <= usize::MAX as u64);
        let months_since_deflationary_phase_started: usize = months_since_deflationary_phase_started as usize;
        if months_since_deflationary_phase_started >= SUBSIDY_BY_MONTH_TABLE.len() {
            *SUBSIDY_BY_MONTH_TABLE.last().unwrap()
        } else {
            SUBSIDY_BY_MONTH_TABLE[months_since_deflationary_phase_started]
        }
    }
}

/*
    Mainnet subsidy-by-month table. Deterministic integer schedule for the
    Zyanya mainnet smooth geometric decay:
        month_subsidy[m] = round(5_000_000_000 * (1 - 0.004847033515)^m)
    for m = 0..727, computed with exact rational arithmetic (no floating
    point). This replaces the previous f64::powf computation, which was not
    bit-identical across platforms/compilers and could cause a consensus fork.
    These values apply to 1 block per second (bps == 1); the manager applies
    `.div_ceil(bps)` at startup exactly like the non-mainnet table.
*/
#[rustfmt::skip]
const MAINNET_SUBSIDY_BY_MONTH_TABLE: [u64; 727] = [
	5000000000, 4975764832, 4951647134, 4927646334, 4903761867, 4879993169, 4856339678, 4832800837, 4809376090, 4786064883, 4762866666, 4739780891, 4716807014, 4693944493, 4671192786, 4648551358, 4626019674, 4603597202, 4581283412, 4559077778, 4536979775, 4514988882, 4493104579, 4471326351, 4449653682,
	4428086062, 4406622980, 4385263931, 4364008410, 4342855915, 4321805946, 4300858008, 4280011605, 4259266246, 4238621439, 4218076699, 4197631540, 4177285479, 4157038037, 4136888734, 4116837096, 4096882648, 4077024921, 4057263444, 4037597752, 4018027381, 3998551867, 3979170752, 3959883578, 3940689890,
	3921589234, 3902581160, 3883665218, 3864840962, 3846107949, 3827465735, 3808913880, 3790451947, 3772079499, 3753796103, 3735601328, 3717494743, 3699475921, 3681544438, 3663699868, 3645941792, 3628269790, 3610683445, 3593182341, 3575766066, 3558434208, 3541186358, 3524022109, 3506941056, 3489942795,
	3473026925, 3456193048, 3439440764, 3422769679, 3406179400, 3389669534, 3373239692, 3356889487, 3340618531, 3324426441, 3308312834, 3292277331, 3276319553, 3260439122, 3244635664, 3228908806, 3213258177, 3197683407, 3182184129, 3166759975, 3151410584, 3136135591, 3120934637, 3105807362, 3090753410,
	3075772424, 3060864052, 3046027941, 3031263742, 3016571105, 3001949684, 2987399133, 2972919109, 2958509271, 2944169277, 2929898790, 2915697472, 2901564989, 2887501006, 2873505192, 2859577216, 2845716750, 2831923465, 2818197037, 2804537142, 2790943456, 2777415660, 2763953433, 2750556458, 2737224419,
	2723957000, 2710753889, 2697614774, 2684539345, 2671527293, 2658578311, 2645692092, 2632868334, 2620106733, 2607406988, 2594768799, 2582191868, 2569675897, 2557220592, 2544825658, 2532490803, 2520215735, 2508000165, 2495843804, 2483746365, 2471707563, 2459727114, 2447804734, 2435940143, 2424133059,
	2412383205, 2400690303, 2389054076, 2377474251, 2365950554, 2354482712, 2343070456, 2331713515, 2320411621, 2309164508, 2297971910, 2286833563, 2275749205, 2264718572, 2253741405, 2242817445, 2231946434, 2221128114, 2210362232, 2199648532, 2188986762, 2178376670, 2167818005, 2157310519, 2146853962,
	2136448089, 2126092654, 2115787411, 2105532119, 2095326534, 2085170416, 2075063525, 2065005623, 2054996471, 2045035835, 2035123477, 2025259166, 2015442667, 2005673748, 1995952181, 1986277733, 1976650179, 1967069289, 1957534838, 1948046601, 1938604354, 1929207874, 1919856939, 1910551328, 1901290821,
	1892075201, 1882904249, 1873777749, 1864695486, 1855657244, 1846662811, 1837711975, 1828804523, 1819940246, 1811118935, 1802340381, 1793604377, 1784910716, 1776259194, 1767649606, 1759081749, 1750555421, 1742070420, 1733626547, 1725223601, 1716861384, 1708539699, 1700258350, 1692017141, 1683815877,
	1675654365, 1667532412, 1659449827, 1651406418, 1643401996, 1635436371, 1627509356, 1619620764, 1611770408, 1603958102, 1596183664, 1588446908, 1580747653, 1573085716, 1565460917, 1557873075, 1550322012, 1542807549, 1535329509, 1527887716, 1520481993, 1513112166, 1505778060, 1498479504, 1491216323,
	1483988348, 1476795406, 1469637330, 1462513948, 1455425094, 1448370600, 1441350299, 1434364026, 1427411615, 1420492903, 1413607727, 1406755923, 1399937330, 1393151786, 1386399133, 1379679210, 1372991859, 1366336921, 1359714240, 1353123660, 1346565024, 1340038178, 1333542968, 1327079241, 1320646843,
	1314245624, 1307875431, 1301536115, 1295227526, 1288949515, 1282701933, 1276484634, 1270297470, 1264140296, 1258012965, 1251915334, 1245847259, 1239808595, 1233799201, 1227818935, 1221867656, 1215945222, 1210051495, 1204186335, 1198349603, 1192541163, 1186760876, 1181008606, 1175284218, 1169587576,
	1163918546, 1158276993, 1152662786, 1147075791, 1141515876, 1135982910, 1130476763, 1124997304, 1119544405, 1114117935, 1108717768, 1103343776, 1097995832, 1092673809, 1087377583, 1082107027, 1076862018, 1071642432, 1066448145, 1061279035, 1056134980, 1051015859, 1045921549, 1040851933, 1035806888,
	1030786298, 1025790042, 1020818003, 1015870064, 1010946108, 1006046018, 1001169680, 996316977, 991487795, 986682020, 981899539, 977140239, 972404008, 967690733, 963000304, 958332609, 953687539, 949064983, 944464833, 939886981, 935331317, 930797735, 926286127, 921796387, 917328409,
	912882088, 908457317, 904053994, 899672014, 895311274, 890971670, 886653101, 882355463, 878078657, 873822580, 869587133, 865372215, 861177727, 857003569, 852849644, 848715854, 844602099, 840508285, 836434313, 832380088, 828345514, 824330495, 820334938, 816358747, 812401828,
	808464090, 804545437, 800645778, 796765021, 792903075, 789059847, 785235247, 781429186, 777641572, 773872318, 770121332, 766388529, 762673818, 758977112, 755298325, 751637368, 747994157, 744368604, 740760625, 737170133, 733597045, 730041275, 726502741, 722981357, 719477043,
	715989713, 712519287, 709065682, 705628817, 702208611, 698804982, 695417851, 692047137, 688692761, 685354645, 682032708, 678726872, 675437060, 672163194, 668905197, 665662991, 662436500, 659225648, 656030359, 652850558, 649686170, 646537119, 643403332, 640284734, 637181253,
	634092814, 631019345, 627960773, 624917026, 621888032, 618873720, 615874018, 612888856, 609918164, 606961870, 604019905, 601092201, 598178687, 595279294, 592393956, 589522602, 586665167, 583821581, 580991778, 578175691, 575373254, 572584401, 569809065, 567047182, 564298685,
	561563510, 558841593, 556132869, 553437275, 550754745, 548085219, 545428631, 542784920, 540154024, 537535879, 534930425, 532337599, 529757341, 527189589, 524634284, 522091364, 519560769, 517042441, 514536319, 512042344, 509560458, 507090601, 504632716, 502186744, 499752628,
	497330311, 494919734, 492520841, 490133576, 487757882, 485393704, 483040984, 480699668, 478369701, 476051027, 473743592, 471447340, 469162219, 466888174, 464625152, 462373098, 460131960, 457901685, 455682220, 453473513, 451275512, 449088165, 446911419, 444745224, 442589529,
	440444283, 438309435, 436184934, 434070731, 431966776, 429873019, 427789410, 425715900, 423652441, 421598983, 419555479, 417521879, 415498137, 413484204, 411480032, 409485574, 407500784, 405525614, 403560018, 401603949, 399657361, 397720208, 395792445, 393874026, 391964905,
	390065038, 388174380, 386292886, 384420511, 382557212, 380702944, 378857665, 377021329, 375193894, 373375316, 371565554, 369764563, 367972302, 366188728, 364413799, 362647473, 360889708, 359140464, 357399698, 355667370, 353943438, 352227862, 350520602, 348821617, 347130867,
	345448312, 343773912, 342107629, 340449421, 338799252, 337157080, 335522869, 333896578, 332278170, 330667607, 329064850, 327469861, 325882604, 324303040, 322731132, 321166844, 319610137, 318060976, 316519324, 314985144, 313458401, 311939057, 310427078, 308922428, 307425071,
	305934971, 304452094, 302976404, 301507868, 300046449, 298592114, 297144828, 295704557, 294271267, 292844924, 291425495, 290012946, 288607243, 287208354, 285816246, 284430885, 283052239, 281680275, 280314962, 278956265, 277604155, 276258598, 274919564, 273587019, 272260934,
	270941276, 269628015, 268321119, 267020557, 265726300, 264438315, 263156574, 261881045, 260611699, 259348505, 258091434, 256840457, 255595542, 254356662, 253123787, 251896887, 250675935, 249460900, 248251755, 247048470, 245851018, 244659370, 243473498, 242293374, 241118969,
	239950258, 238787211, 237629801, 236478002, 235331785, 234191124, 233055991, 231926361, 230802206, 229683500, 228570217, 227462329, 226359812, 225262638, 224170783, 223084219, 222002923, 220926867, 219856027, 218790378, 217729893, 216674549, 215624320, 214579182, 213539110,
	212504078, 211474064, 210449042, 209428988, 208413879, 207403690, 206398397, 205397978, 204402407, 203411661, 202425718, 201444554, 200468145, 199496470, 198529504, 197567224, 196609609, 195656636, 194708282, 193764524, 192825341, 191890710, 190960609, 190035017, 189113911,
	188197269, 187285071, 186377294, 185473917, 184574919, 183680278, 182789973, 181903984, 181022290, 180144868, 179271700, 178402764, 177538040, 176677507, 175821145, 174968935, 174120854, 173276885, 172437006, 171601198, 170769441, 169941716, 169118003, 168298282, 167482535,
	166670741, 165862882, 165058940, 164258893, 163462725, 162670416, 161881947, 161097299, 160316455, 159539396, 158766103, 157996559, 157230744, 156468641, 155710233, 154955500, 154204426, 153456992, 152713180, 151972974, 151236356, 150503309, 149773814, 149047855, 148325415,
	147606477, 146891024,
];

/*
    This table was pre-calculated by calling `calcDeflationaryPeriodBlockSubsidyFloatCalc` (in zyanyad-go) for all months until reaching 0 subsidy.
    To regenerate this table, run `TestBuildSubsidyTable` in coinbasemanager_test.go (note the `deflationaryPhaseBaseSubsidy` therein).
    These values apply to 1 block per second.
*/
#[rustfmt::skip]
const SUBSIDY_BY_MONTH_TABLE: [u64; 727] = [
	1200000000, 1175000000, 1150000000, 1125000000, 1100000000, 1075000000, 1050000000, 1025000000, 1000000000, 975000000, 950000000, 925000000, 900000000, 875000000, 850000000, 825000000, 800000000, 775000000, 750000000, 725000000, 700000000, 675000000, 650000000, 625000000, 600000000,
	587500000, 575000000, 562500000, 550000000, 537500000, 525000000, 512500000, 500000000, 487500000, 475000000, 462500000, 450000000, 437500000, 425000000, 412500000, 400000000, 387500000, 375000000, 362500000, 350000000, 337500000, 325000000, 312500000, 300000000, 293750000,
	287500000, 281250000, 275000000, 268750000, 262500000, 256250000, 250000000, 243750000, 237500000, 231250000, 225000000, 218750000, 212500000, 206250000, 200000000, 193750000, 187500000, 181250000, 175000000, 168750000, 162500000, 156250000, 150000000, 146875000, 143750000,
	140625000, 137500000, 134375000, 131250000, 128125000, 125000000, 121875000, 118750000, 115625000, 112500000, 109375000, 106250000, 103125000, 100000000, 96875000, 93750000, 90625000, 87500000, 84375000, 81250000, 78125000, 75000000, 73437500, 71875000, 70312500,
	68750000, 67187500, 65625000, 64062500, 62500000, 60937500, 59375000, 57812500, 56250000, 54687500, 53125000, 51562500, 50000000, 48437500, 46875000, 45312500, 43750000, 42187500, 40625000, 39062500, 37500000, 36718750, 35937500, 35156250, 34375000,
	33593750, 32812500, 32031250, 31250000, 30468750, 29687500, 28906250, 28125000, 27343750, 26562500, 25781250, 25000000, 24218750, 23437500, 22656250, 21875000, 21093750, 20312500, 19531250, 18750000, 18359375, 17968750, 17578125, 17187500, 16796875,
	16406250, 16015625, 15625000, 15234375, 14843750, 14453125, 14062500, 13671875, 13281250, 12890625, 12500000, 12109375, 11718750, 11328125, 10937500, 10546875, 10156250, 9765625, 9375000, 9179687, 8984375, 8789062, 8593750, 8398437, 8203125,
	8007812, 7812500, 7617187, 7421875, 7226562, 7031250, 6835937, 6640625, 6445312, 6250000, 6054687, 5859375, 5664062, 5468750, 5273437, 5078125, 4882812, 4687500, 4589843, 4492187, 4394531, 4296875, 4199218, 4101562, 4003906,
	3906250, 3808593, 3710937, 3613281, 3515625, 3417968, 3320312, 3222656, 3125000, 3027343, 2929687, 2832031, 2734375, 2636718, 2539062, 2441406, 2343750, 2294921, 2246093, 2197265, 2148437, 2099609, 2050781, 2001953, 1953125,
	1904296, 1855468, 1806640, 1757812, 1708984, 1660156, 1611328, 1562500, 1513671, 1464843, 1416015, 1367187, 1318359, 1269531, 1220703, 1171875, 1147460, 1123046, 1098632, 1074218, 1049804, 1025390, 1000976, 976562, 952148,
	927734, 903320, 878906, 854492, 830078, 805664, 781250, 756835, 732421, 708007, 683593, 659179, 634765, 610351, 585937, 573730, 561523, 549316, 537109, 524902, 512695, 500488, 488281, 476074, 463867,
	451660, 439453, 427246, 415039, 402832, 390625, 378417, 366210, 354003, 341796, 329589, 317382, 305175, 292968, 286865, 280761, 274658, 268554, 262451, 256347, 250244, 244140, 238037, 231933, 225830,
	219726, 213623, 207519, 201416, 195312, 189208, 183105, 177001, 170898, 164794, 158691, 152587, 146484, 143432, 140380, 137329, 134277, 131225, 128173, 125122, 122070, 119018, 115966, 112915, 109863,
	106811, 103759, 100708, 97656, 94604, 91552, 88500, 85449, 82397, 79345, 76293, 73242, 71716, 70190, 68664, 67138, 65612, 64086, 62561, 61035, 59509, 57983, 56457, 54931, 53405,
	51879, 50354, 48828, 47302, 45776, 44250, 42724, 41198, 39672, 38146, 36621, 35858, 35095, 34332, 33569, 32806, 32043, 31280, 30517, 29754, 28991, 28228, 27465, 26702, 25939,
	25177, 24414, 23651, 22888, 22125, 21362, 20599, 19836, 19073, 18310, 17929, 17547, 17166, 16784, 16403, 16021, 15640, 15258, 14877, 14495, 14114, 13732, 13351, 12969, 12588,
	12207, 11825, 11444, 11062, 10681, 10299, 9918, 9536, 9155, 8964, 8773, 8583, 8392, 8201, 8010, 7820, 7629, 7438, 7247, 7057, 6866, 6675, 6484, 6294, 6103,
	5912, 5722, 5531, 5340, 5149, 4959, 4768, 4577, 4482, 4386, 4291, 4196, 4100, 4005, 3910, 3814, 3719, 3623, 3528, 3433, 3337, 3242, 3147, 3051, 2956,
	2861, 2765, 2670, 2574, 2479, 2384, 2288, 2241, 2193, 2145, 2098, 2050, 2002, 1955, 1907, 1859, 1811, 1764, 1716, 1668, 1621, 1573, 1525, 1478, 1430,
	1382, 1335, 1287, 1239, 1192, 1144, 1120, 1096, 1072, 1049, 1025, 1001, 977, 953, 929, 905, 882, 858, 834, 810, 786, 762, 739, 715, 691,
	667, 643, 619, 596, 572, 560, 548, 536, 524, 512, 500, 488, 476, 464, 452, 441, 429, 417, 405, 393, 381, 369, 357, 345, 333,
	321, 309, 298, 286, 280, 274, 268, 262, 256, 250, 244, 238, 232, 226, 220, 214, 208, 202, 196, 190, 184, 178, 172, 166, 160,
	154, 149, 143, 140, 137, 134, 131, 128, 125, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 92, 89, 86, 83, 80, 77,
	74, 71, 70, 68, 67, 65, 64, 62, 61, 59, 58, 56, 55, 53, 52, 50, 49, 47, 46, 44, 43, 41, 40, 38, 37,
	35, 35, 34, 33, 32, 32, 31, 30, 29, 29, 28, 27, 26, 26, 25, 24, 23, 23, 22, 21, 20, 20, 19, 18, 17,
	17, 17, 16, 16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 11, 11, 11, 10, 10, 10, 9, 9, 8, 8,
	8, 8, 8, 8, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4,
	4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
	2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
	1, 0,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::MAINNET_PARAMS;
    use zyanya_consensus_core::{
        config::params::{Params, TESTNET11_PARAMS},
        constants::SOMPI_PER_ZYANYA,
        network::NetworkId,
        tx::scriptvec,
    };

    #[test]
    fn calc_high_bps_total_rewards_delta() {
        const SECONDS_PER_MONTH: u64 = 2629800;

        let legacy_cbm = create_legacy_manager();
        let pre_deflationary_rewards = legacy_cbm.pre_deflationary_phase_base_subsidy * legacy_cbm.deflationary_phase_daa_score;
        let total_rewards: u64 = pre_deflationary_rewards + SUBSIDY_BY_MONTH_TABLE.iter().map(|x| x * SECONDS_PER_MONTH).sum::<u64>();
        let testnet_11_bps = TESTNET11_PARAMS.bps();
        let total_high_bps_rewards_rounded_up: u64 = pre_deflationary_rewards
            + SUBSIDY_BY_MONTH_TABLE.iter().map(|x| (x.div_ceil(testnet_11_bps) * testnet_11_bps) * SECONDS_PER_MONTH).sum::<u64>();

        let cbm = create_manager(&TESTNET11_PARAMS);
        let total_high_bps_rewards: u64 =
            pre_deflationary_rewards + cbm.subsidy_by_month_table.iter().map(|x| x * cbm.blocks_per_month).sum::<u64>();
        assert_eq!(total_high_bps_rewards_rounded_up, total_high_bps_rewards, "subsidy adjusted to bps must be rounded up");

        let delta = total_high_bps_rewards as i64 - total_rewards as i64;

        println!("Total rewards: {} sompi => {} ZYAN", total_rewards, total_rewards / SOMPI_PER_ZYANYA);
        println!("Total high bps rewards: {} sompi => {} ZYAN", total_high_bps_rewards, total_high_bps_rewards / SOMPI_PER_ZYANYA);
        println!("Delta: {} sompi => {} ZYAN", delta, delta / SOMPI_PER_ZYANYA as i64);
    }

    #[test]
    fn subsidy_by_month_table_test() {
        let cbm = create_legacy_manager();
        cbm.subsidy_by_month_table.iter().enumerate().for_each(|(i, x)| {
            assert_eq!(SUBSIDY_BY_MONTH_TABLE[i], *x, "for 1 BPS, const table and precomputed values must match");
        });

        for network_id in NetworkId::iter() {
            if network_id.is_mainnet() {
                continue;
            }
            let cbm = create_manager(&network_id.into());
            cbm.subsidy_by_month_table.iter().enumerate().for_each(|(i, x)| {
                assert_eq!(
                    SUBSIDY_BY_MONTH_TABLE[i].div_ceil(cbm.bps()),
                    *x,
                    "{}: locally computed and precomputed values must match",
                    network_id
                );
            });
        }
    }

    #[test]
    fn subsidy_test() {
        const PRE_DEFLATIONARY_PHASE_BASE_SUBSIDY: u64 = 1500000000;
        const DEFLATIONARY_PHASE_INITIAL_SUBSIDY: u64 = 1200000000;
        const SECONDS_PER_MONTH: u64 = 2629800;
        const SECONDS_PER_HALVING: u64 = SECONDS_PER_MONTH * 24;

        for network_id in NetworkId::iter() {
            if network_id.is_mainnet() {
                continue;
            }
            let params = &network_id.into();
            let cbm = create_manager(params);

            let pre_deflationary_phase_base_subsidy = cbm.calc_block_subsidy(1);
            let deflationary_phase_initial_subsidy = DEFLATIONARY_PHASE_INITIAL_SUBSIDY / params.bps();
            let blocks_per_halving = SECONDS_PER_HALVING * params.bps();

            struct Test {
                name: &'static str,
                daa_score: u64,
                expected: u64,
            }

            let tests = vec![
                Test { name: "first mined block", daa_score: 1, expected: pre_deflationary_phase_base_subsidy },
                Test {
                    name: "before deflationary phase",
                    daa_score: params.deflationary_phase_daa_score - 1,
                    expected: pre_deflationary_phase_base_subsidy,
                },
                Test {
                    name: "start of deflationary phase",
                    daa_score: params.deflationary_phase_daa_score,
                    expected: deflationary_phase_initial_subsidy,
                },
                Test {
                    name: "after 2 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving,
                    expected: deflationary_phase_initial_subsidy / 2,
                },
                Test {
                    name: "after 4 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 2,
                    expected: deflationary_phase_initial_subsidy / 4,
                },
                Test {
                    name: "after 8 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 4,
                    expected: deflationary_phase_initial_subsidy / 16,
                },
                Test {
                    name: "after 16 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 8,
                    expected: deflationary_phase_initial_subsidy / 256,
                },
                Test {
                    name: "after 32 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 16,
                    expected: deflationary_phase_initial_subsidy / 65536,
                },
                Test {
                    name: "just before subsidy depleted",
                    daa_score: params.deflationary_phase_daa_score + (blocks_per_halving / 24 * 725),
                    expected: 1,
                },
                Test {
                    name: "after subsidy depleted",
                    daa_score: params.deflationary_phase_daa_score + (blocks_per_halving / 24 * 726),
                    expected: 0,
                },
            ];

            for t in tests {
                assert_eq!(cbm.calc_block_subsidy(t.daa_score), t.expected, "{} test '{}' failed", network_id, t.name);
                if params.bps() == 1 {
                    assert_eq!(cbm.legacy_calc_block_subsidy(t.daa_score), t.expected, "{} test '{}' failed", network_id, t.name);
                }
            }
        }
    }

    #[test]
    fn mainnet_smooth_decay_subsidy_test() {
        let cbm = create_manager(&MAINNET_PARAMS);
        assert_eq!(cbm.calc_block_subsidy(1), 5_000_000_000, "Year 1 pre-deflationary subsidy should be 50 ZYAN");
        assert_eq!(cbm.calc_block_subsidy(MAINNET_PARAMS.deflationary_phase_daa_score - 1), 5_000_000_000);
        assert_eq!(cbm.calc_block_subsidy(MAINNET_PARAMS.deflationary_phase_daa_score), 5_000_000_000);

        let month_1_daa = MAINNET_PARAMS.deflationary_phase_daa_score + cbm.blocks_per_month;
        assert_eq!(cbm.calc_block_subsidy(month_1_daa), 4_975_764_832);

        let pre_deflationary_total = MAINNET_PARAMS.deflationary_phase_daa_score * 5_000_000_000;
        let deflationary_monthly_sum: u64 = cbm.subsidy_by_month_table.iter().map(|&s| s * cbm.blocks_per_month).sum();
        let total_supply_sompi = pre_deflationary_total + deflationary_monthly_sum;
        let total_supply_zyan = total_supply_sompi / SOMPI_PER_ZYANYA;
        println!("Mainnet Total Supply (727 months table sum): {} ZYAN", total_supply_zyan);
        assert!(total_supply_zyan >= 27_500_000_000 && total_supply_zyan <= 28_700_000_000, "Mainnet total supply must approach ~28.7B ZYAN");
    }

    #[test]
    fn payload_serialization_test() {
        let cbm = create_manager(&MAINNET_PARAMS);

        let script_data = [33u8, 255];
        let extra_data = [2u8, 3];
        let data = CoinbaseData {
            blue_score: 56,
            subsidy: 1200000000,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&script_data)),
                extra_data: &extra_data as &[u8],
            },
        };

        let payload = cbm.serialize_coinbase_payload(&data).unwrap();
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        assert_eq!(data, deserialized_data);

        // Test an actual mainnet payload
        let payload_hex =
            "b612c90100000000041a763e07000000000022202b32443ff740012157716d81216d09aebc39e5493c93a7181d92cb756c02c560ac302e31322e382f";
        let mut payload = vec![0u8; payload_hex.len() / 2];
        faster_hex::hex_decode(payload_hex.as_bytes(), &mut payload).unwrap();
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        let expected_data = CoinbaseData {
            blue_score: 29954742,
            subsidy: 31112698372,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(
                    0,
                    scriptvec![
                        32, 43, 50, 68, 63, 247, 64, 1, 33, 87, 113, 109, 129, 33, 109, 9, 174, 188, 57, 229, 73, 60, 147, 167, 24,
                        29, 146, 203, 117, 108, 2, 197, 96, 172,
                    ],
                ),
                extra_data: &[48u8, 46, 49, 50, 46, 56, 47] as &[u8],
            },
        };
        assert_eq!(expected_data, deserialized_data);
    }

    #[test]
    fn modify_payload_test() {
        let cbm = create_manager(&MAINNET_PARAMS);

        let script_data = [33u8, 255];
        let extra_data = [2u8, 3, 23, 98];
        let data = CoinbaseData {
            blue_score: 56345,
            subsidy: 1200000000,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&script_data)),
                extra_data: &extra_data,
            },
        };

        let data2 = CoinbaseData {
            blue_score: data.blue_score,
            subsidy: data.subsidy,
            miner_data: MinerData {
                // Modify only miner data
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&[33u8, 255, 33])),
                extra_data: &[2u8, 3, 23, 98, 34, 34] as &[u8],
            },
        };

        let mut payload = cbm.serialize_coinbase_payload(&data).unwrap();
        payload = cbm.modify_coinbase_payload(payload, &data2.miner_data).unwrap(); // Update the payload with the modified miner data
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        assert_eq!(data2, deserialized_data);
    }

    #[test]
    fn expected_coinbase_transaction_vesting_split_test() {
        let cbm = create_manager(&MAINNET_PARAMS);
        let script_data = [33u8, 255];
        let miner_script = ScriptPublicKey::new(0, ScriptVec::from_slice(&script_data));
        let miner_data = MinerData {
            script_public_key: miner_script.clone(),
            extra_data: vec![],
        };

        let blue_hash = zyanya_hashes::Hash::from_u64_word(1);
        let mut mergeset_blues = Vec::new();
        mergeset_blues.push(blue_hash);

        let ghostdag_data = GhostdagData::new(
            0,
            0.into(),
            blue_hash,
            mergeset_blues.into(),
            Vec::new().into(),
            BlockHashMap::default().into(),
        );

        let total_subsidy = 5_000_000_000u64; // 50 ZYAN
        let mut mergeset_rewards = BlockHashMap::default();
        mergeset_rewards.insert(
            blue_hash,
            BlockRewardData::new(total_subsidy, 0, miner_script.clone()),
        );

        let mergeset_non_daa = BlockHashSet::default();

        let template = cbm
            .expected_coinbase_transaction(1, miner_data, &ghostdag_data, &mergeset_rewards, &mergeset_non_daa)
            .unwrap();

        // Should have 13 outputs: 1 liquid + 12 monthly vested outputs
        assert_eq!(template.tx.outputs.len(), 13);

        // Output 0 is liquid 50% = 2_500_000_000 (and has unlocked miner script)
        assert_eq!(template.tx.outputs[0].value, 2_500_000_000);
        assert_eq!(template.tx.outputs[0].script_public_key, miner_script);

        // Outputs 1..13 sum to 2_500_000_000 (50% vested, each with CSV time-lock)
        let vested_sum: u64 = template.tx.outputs[1..].iter().map(|o| o.value).sum();
        assert_eq!(vested_sum, 2_500_000_000);

        for i in 0..12 {
            let expected_lock = (i as u64 + 1) * cbm.blocks_per_month;
            let expected_spk = create_csv_locked_script(&miner_script, expected_lock);
            assert_eq!(template.tx.outputs[i + 1].script_public_key, expected_spk);
        }

        // Total transaction output value must equal total subsidy
        let total_sum: u64 = template.tx.outputs.iter().map(|o| o.value).sum();
        assert_eq!(total_sum, total_subsidy);
    }

    #[test]
    fn csv_vested_output_spending_test() {
        use zyanya_consensus_core::{
            hashing::sighash::SigHashReusedValuesUnsync,
            tx::{PopulatedTransaction, TransactionInput, TransactionOutpoint, UtxoEntry},
        };
        use zyanya_txscript::{caches::Cache, opcodes::codes::OpTrue, SigCacheKey, TxScriptEngine};

        let cbm = create_manager(&MAINNET_PARAMS);
        let base_script = vec![OpTrue];
        let base_spk = ScriptPublicKey::new(0, ScriptVec::from_slice(&base_script));

        let lock_blocks = cbm.blocks_per_month; // month 1
        let vested_spk = create_csv_locked_script(&base_spk, lock_blocks);

        // 1. Test TxScriptEngine execution for liquid vs vested outputs
        let input = TransactionInput {
            previous_outpoint: TransactionOutpoint::new(zyanya_hashes::Hash::from_u64_word(1), 0),
            signature_script: vec![],
            sequence: lock_blocks,
            sig_op_count: 0,
        };

        let tx = Transaction::new(1, vec![input.clone()], vec![], 0, Default::default(), 0, vec![]);
        let utxo_entry = UtxoEntry::new(1_000_000, vested_spk.clone(), 100, true);
        let populated_tx = PopulatedTransaction::new(&tx, vec![utxo_entry.clone()]);

        let sig_cache = Cache::<SigCacheKey, bool>::new(10_000);
        let reused_values = SigHashReusedValuesUnsync::new();

        // Valid sequence (sequence == lock_blocks) should succeed
        let mut vm = TxScriptEngine::from_transaction_input(
            &populated_tx,
            &input,
            0,
            &utxo_entry,
            &reused_values,
            &sig_cache,
            false,
            false,
        );
        assert!(vm.execute().is_ok());

        // Invalid sequence (sequence < lock_blocks) should fail
        let mut invalid_input = input.clone();
        invalid_input.sequence = lock_blocks - 1;
        let invalid_tx = Transaction::new(1, vec![invalid_input.clone()], vec![], 0, Default::default(), 0, vec![]);
        let invalid_populated = PopulatedTransaction::new(&invalid_tx, vec![utxo_entry.clone()]);
        let mut vm_invalid = TxScriptEngine::from_transaction_input(
            &invalid_populated,
            &invalid_input,
            0,
            &utxo_entry,
            &reused_values,
            &sig_cache,
            false,
            false,
        );
        assert!(vm_invalid.execute().is_err());

        // Disabled sequence (sequence has bit 63 set) should fail for CSV opcode
        let mut disabled_input = input.clone();
        disabled_input.sequence = u64::MAX;
        let disabled_tx = Transaction::new(1, vec![disabled_input.clone()], vec![], 0, Default::default(), 0, vec![]);
        let disabled_populated = PopulatedTransaction::new(&disabled_tx, vec![utxo_entry.clone()]);
        let mut vm_disabled = TxScriptEngine::from_transaction_input(
            &disabled_populated,
            &disabled_input,
            0,
            &utxo_entry,
            &reused_values,
            &sig_cache,
            false,
            false,
        );
        assert!(vm_disabled.execute().is_err());
    }

    fn create_manager(params: &Params) -> CoinbaseManager {
        CoinbaseManager::new(
            params.coinbase_payload_script_public_key_max_len,
            params.max_coinbase_payload_len,
            params.deflationary_phase_daa_score,
            params.pre_deflationary_phase_base_subsidy,
            params.target_time_per_block,
        )
    }

    /// Return a CoinbaseManager with legacy golang 1 BPS properties
    fn create_legacy_manager() -> CoinbaseManager {
        CoinbaseManager::new(150, 204, 604800, 1500000000, 1000)
    }
}
