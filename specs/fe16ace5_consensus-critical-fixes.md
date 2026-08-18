# Remediation Plan — Consensus CRITICAL Fixes (F-C-06, F-C-07, F-C-08)

**Session:** fe16ace5
**Date:** 2026-08-18
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md` (and `audit_reports/phase2_findings.md` for the original C-01/C-02/C-03 details).

## Objective

Remediate three CRITICAL consensus-integrity findings in the Zyanya blockchain
(a Kaspa/rusty-spectre fork):

| ID | Title | Root cause |
|----|-------|-----------|
| F-C-06 | `f64::powf` in consensus-critical subsidy schedule | Non-deterministic floating point in `CoinbaseManager::new` |
| F-C-07 | Coinbase output-count limit too small for 13× vesting split | Limit derived from `ghostdag_k + 2` instead of `mergeset_size_limit` |
| F-C-08 | Transaction mass calculation overflow enables block-mass bypass | Plain `*`/`+`/`.sum()` in mass computation can wrap |

All three are consensus-critical: any divergence between nodes (F-C-06), any
organic chain halt (F-C-07), or any mass-limit bypass (F-C-08) is a fork/DoS
vector. The fixes below are **integer-only, deterministic, and overflow-safe**.

---

## Files to modify (summary)

1. `consensus/src/processes/coinbase.rs` — F-C-06
2. `consensus/src/processes/transaction_validator/mod.rs` — F-C-07 (add field + ctor params)
3. `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` — F-C-07 (limit + 1 test call site)
4. `consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs` — F-C-07 (8 test call sites)
5. `consensus/src/consensus/services.rs` — F-C-07 (1 production call site)
6. `consensus/core/src/mass/mod.rs` — F-C-08

---

## Fix 1 — F-C-06: Deterministic mainnet subsidy schedule

**File:** `consensus/src/processes/coinbase.rs`

### Current code (lines ~86-95)

```rust
let subsidy_by_month_table: SubsidyByMonthTable = if deflationary_phase_daa_score == 31_449_600 {
    core::array::from_fn(|i| {
        let m = i as f64;
        let decay_rate = 0.004847033515f64;
        let initial = 5_000_000_000f64;
        let month_subsidy = (initial * (1.0 - decay_rate).powf(m)).round() as u64;
        month_subsidy.div_ceil(bps)
    })
} else {
    core::array::from_fn(|i| SUBSIDY_BY_MONTH_TABLE[i].div_ceil(bps))
};
```

### Required change

Replace the float branch with a precomputed constant integer table, mirroring
the existing non-mainnet `SUBSIDY_BY_MONTH_TABLE` pattern:

```rust
let subsidy_by_month_table: SubsidyByMonthTable = if deflationary_phase_daa_score == 31_449_600 {
    core::array::from_fn(|i| MAINNET_SUBSIDY_BY_MONTH_TABLE[i].div_ceil(bps))
} else {
    core::array::from_fn(|i| SUBSIDY_BY_MONTH_TABLE[i].div_ceil(bps))
};
```

Keep the `.div_ceil(bps)` call — for mainnet `bps == 1` (target_time_per_block
== 1000) so it is an identity, but keeping it preserves the existing rounding
semantics and consistency with the non-mainnet path.

Also update the comment block above the table computation to state that the
mainnet table is a precomputed integer constant (deterministic across all
platforms), not a runtime float computation.

### Add the constant

Add `MAINNET_SUBSIDY_BY_MONTH_TABLE` immediately before the existing
`SUBSIDY_BY_MONTH_TABLE` constant (near the bottom of the file, after the
`impl CoinbaseManager` block). The table is the exact integer schedule
`round(5_000_000_000 * (1 - 0.004847033515)^m)` for `m = 0..727`, computed
with exact rational arithmetic (no floating point). It is 727 entries, same
size as `SUBSIDY_BY_MONTH_TABLE`.

```rust
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
```

**Checksum for verification:** the table has exactly 727 entries; `sum ==
1_001_400_330_135`; `[0] == 5_000_000_000`; `[1] == 4_975_764_832`; `[2] ==
4_951_647_134`; `[726] == 146_891_024`. These match the existing
`mainnet_smooth_decay_subsidy_test` expectations (month-1 subsidy
`4_975_764_832` and total supply in `[27.5B, 28.7B]` ZYAN).

### Optional (recommended) regression test

Extend `mainnet_smooth_decay_subsidy_test` in `coinbase.rs` with golden
assertions for a few more table entries (e.g. month 0, 2, 12, 24, 726) to lock
the table in. Not required for `cargo check` to pass.

---

## Fix 2 — F-C-07: Coinbase output-count limit

The coinbase output limit must be derived from `mergeset_size_limit` (the real
bound on payable DAA-window blues), not `ghostdag_k + 2`. On mainnet this
changes the limit from `(18 + 2) * 13 = 260` to `(180 + 1) * 13 = 2353`.

### 2a. `consensus/src/processes/transaction_validator/mod.rs`

Add a `mergeset_size_limit: u64` field to `TransactionValidator` and thread it
through both constructors.

1. Add the field after `ghostdag_k` in the struct:

```rust
    ghostdag_k: ghostdag::KType,
    mergeset_size_limit: u64,
    coinbase_payload_script_public_key_max_len: u8,
```

2. Add the parameter to `new(...)` after `ghostdag_k`:

```rust
        ghostdag_k: ghostdag::KType,
        mergeset_size_limit: u64,
        coinbase_payload_script_public_key_max_len: u8,
```

and initialize it in the struct literal (after `ghostdag_k,`):

```rust
            ghostdag_k,
            mergeset_size_limit,
            coinbase_payload_script_public_key_max_len,
```

3. Add the parameter to `new_for_tests(...)` after `ghostdag_k`:

```rust
        ghostdag_k: ghostdag::KType,
        mergeset_size_limit: u64,
        coinbase_payload_script_public_key_max_len: u8,
```

and initialize it in the struct literal (after `ghostdag_k,`):

```rust
            ghostdag_k,
            mergeset_size_limit,
            coinbase_payload_script_public_key_max_len,
```

### 2b. `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`

Change the limit computation (line 46):

```rust
        let outputs_limit = (self.ghostdag_k as u64 + 2) * 13;
```

to:

```rust
        let outputs_limit = (self.mergeset_size_limit + 1) * 13;
```

(`+1` accounts for the red-reward block; `* 13` = 1 liquid + 12 vested outputs
per payable blue.)

Update the single test call site in this file (the `new_for_tests` call inside
`validate_tx_in_isolation_test`, around line 215). Insert
`params.mergeset_size_limit,` immediately after `params.ghostdag_k,`:

```rust
        let tv = TransactionValidator::new_for_tests(
            params.max_tx_inputs,
            params.max_tx_outputs,
            params.max_signature_script_len,
            params.max_script_public_key_len,
            params.ghostdag_k,
            params.mergeset_size_limit,
            params.coinbase_payload_script_public_key_max_len,
            params.coinbase_maturity,
            Default::default(),
        );
```

### 2c. `consensus/src/processes/transaction_validator/tx_validation_in_utxo_context.rs`

There are **8** `new_for_tests` call sites in this file (around lines 278, 347,
420, 490, 560, 630, 699, 762). Each currently looks like:

```rust
        let tv = TransactionValidator::new_for_tests(
            params.max_tx_inputs,
            params.max_tx_outputs,
            params.max_signature_script_len,
            params.max_script_public_key_len,
            params.ghostdag_k,
            params.coinbase_payload_script_public_key_max_len,
            params.coinbase_maturity,
            Default::default(),
        );
```

Insert `params.mergeset_size_limit,` immediately after `params.ghostdag_k,` in
**all 8** call sites. (A mechanical edit: every occurrence of
`params.ghostdag_k,\n            params.coinbase_payload_script_public_key_max_len,`
inside a `new_for_tests` call gets the new line inserted between them.)

### 2d. `consensus/src/consensus/services.rs`

Update the production `TransactionValidator::new(...)` call (around line 138).
Insert `params.mergeset_size_limit,` immediately after `params.ghostdag_k,`:

```rust
        let transaction_validator = TransactionValidator::new(
            params.max_tx_inputs,
            params.max_tx_outputs,
            params.max_signature_script_len,
            params.max_script_public_key_len,
            params.ghostdag_k,
            params.mergeset_size_limit,
            params.coinbase_payload_script_public_key_max_len,
            params.coinbase_maturity,
            tx_script_cache_counters,
            mass_calculator.clone(),
            params.storage_mass_activation,
            params.kip10_activation,
            params.payload_activation,
            params.runtime_sig_op_counting,
        );
```

### Note on `coinbase.rs` capacity hint

`consensus/src/processes/coinbase.rs:125` uses
`Vec::with_capacity((ghostdag_data.mergeset_blues.len() + 1) * 13)` — this is
only a preallocation hint and is already correct (it uses the actual blues
count). **No change needed** there.

---

## Fix 3 — F-C-08: Mass calculation overflow protection

**File:** `consensus/core/src/mass/mod.rs`

Replace all plain `+`, `*`, and `.sum::<u64>()` in the compute-mass path with
saturating arithmetic so any overflow saturates at `u64::MAX` (which always
exceeds `max_block_mass = 500_000`, closing the bypass). `saturating_*` is
deterministic and has no panic path.

### 3a. `transaction_estimated_serialized_size`

Replace `size += X` with `size = size.saturating_add(X)` and replace the two
`.sum()` calls with saturating folds:

```rust
pub fn transaction_estimated_serialized_size(tx: &Transaction) -> u64 {
    let mut size: u64 = 0;
    size = size.saturating_add(2); // Tx version (u16)
    size = size.saturating_add(8); // Number of inputs (u64)
    let inputs_size: u64 = tx
        .inputs
        .iter()
        .map(transaction_input_estimated_serialized_size)
        .fold(0u64, |acc, x| acc.saturating_add(x));
    size = size.saturating_add(inputs_size);

    size = size.saturating_add(8); // number of outputs (u64)
    let outputs_size: u64 = tx
        .outputs
        .iter()
        .map(transaction_output_estimated_serialized_size)
        .fold(0u64, |acc, x| acc.saturating_add(x));
    size = size.saturating_add(outputs_size);

    size = size.saturating_add(8); // lock time (u64)
    size = size.saturating_add(SUBNETWORK_ID_SIZE as u64);
    size = size.saturating_add(8); // gas (u64)
    size = size.saturating_add(HASH_SIZE as u64); // payload hash

    size = size.saturating_add(8); // length of the payload (u64)
    size = size.saturating_add(tx.payload.len() as u64);
    size
}
```

### 3b. `transaction_input_estimated_serialized_size`

```rust
fn transaction_input_estimated_serialized_size(input: &TransactionInput) -> u64 {
    let mut size = 0;
    size = size.saturating_add(outpoint_estimated_serialized_size());

    size = size.saturating_add(8); // length of signature script (u64)
    size = size.saturating_add(input.signature_script.len() as u64);

    size = size.saturating_add(8); // sequence (uint64)
    size
}
```

### 3c. `transaction_output_estimated_serialized_size`

```rust
pub fn transaction_output_estimated_serialized_size(output: &TransactionOutput) -> u64 {
    let mut size: u64 = 0;
    size = size.saturating_add(8); // value (u64)
    size = size.saturating_add(2); // output.ScriptPublicKey.Version (u16)
    size = size.saturating_add(8); // length of script public key (u64)
    size = size.saturating_add(output.script_public_key.script().len() as u64);
    size
}
```

### 3d. `calc_tx_compute_mass`

```rust
    pub fn calc_tx_compute_mass(&self, tx: &Transaction) -> u64 {
        if tx.is_coinbase() {
            return 0;
        }

        let size = transaction_estimated_serialized_size(tx);
        let mass_for_size = size.saturating_mul(self.mass_per_tx_byte);
        let total_script_public_key_size: u64 = tx
            .outputs
            .iter()
            .map(|output| 2u64.saturating_add(output.script_public_key.script().len() as u64))
            .fold(0u64, |acc, x| acc.saturating_add(x));
        let total_script_public_key_mass = total_script_public_key_size.saturating_mul(self.mass_per_script_pub_key_byte);

        let total_sigops: u64 = tx
            .inputs
            .iter()
            .map(|input| input.sig_op_count as u64)
            .fold(0u64, |acc, x| acc.saturating_add(x));
        let total_sigops_mass = total_sigops.saturating_mul(self.mass_per_sig_op);

        mass_for_size
            .saturating_add(total_script_public_key_mass)
            .saturating_add(total_sigops_mass)
    }
```

### Do NOT lower the `max_tx_*` params

The audit report's secondary suggestion (lowering `max_tx_inputs`,
`max_tx_outputs`, `max_signature_script_len`, `max_script_public_key_len` in
`consensus/core/src/config/params.rs` from `1_000_000_000` to e.g. `10_000`)
is a **consensus-parameter change** (a soft/hard fork) and is explicitly
deferred to "the next planned HF/SF" in the code comment. **Leave the params
unchanged** for this task; the saturating arithmetic is the complete fix for
the overflow bypass.

---

## Verification

After all edits, run (from `/root/zyanya-audit`):

```bash
source /root/.cargo/env && cargo check -p zyanya-consensus 2>&1
```

> **Important:** the task prompt says `cargo check -p consensus`, but the
> package is named `zyanya-consensus` (there is no package literally named
> `consensus`; `cargo pkgid consensus` fails). Use `-p zyanya-consensus`.
> This also compiles `zyanya-consensus-core` (where the mass fix lives) as a
> dependency. If you want to check the core crate in isolation too, also run
> `cargo check -p zyanya-consensus-core`.

Fix any compilation errors. **Do NOT run a full build or tests** (per the task
instructions). `cargo check` only.

### Expected compile-sensitive points

- `mergeset_size_limit` must be added to **both** constructors and **all 9**
  `new_for_tests` call sites (1 in isolation + 8 in utxo_context) plus the 1
  production `new` call site in `services.rs`. A missed call site is a compile
  error (missing argument).
- The `MAINNET_SUBSIDY_BY_MONTH_TABLE` constant must have exactly 727 entries
  (the type is `[u64; 727]`); a wrong count is a compile error.
- `saturating_add`/`saturating_mul` are inherent `u64` methods — no imports
  needed.

---

## Out of scope (do not touch)

- `consensus/core/src/config/params.rs` (param lowering is a fork, deferred).
- `consensus/src/processes/coinbase.rs` `Vec::with_capacity` hint (already correct).
- Any wallet/mining/rpc code. The `transaction_estimated_serialized_size`
  signature is unchanged, so downstream callers in `mining/` are unaffected.
- Other CRITICAL findings (F-C-01…F-C-05, F-C-09…F-C-16) are handled in
  separate remediation groups.
