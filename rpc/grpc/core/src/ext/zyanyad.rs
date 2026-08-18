use zyanya_notify::{scope::Scope, subscription::Command};

use crate::protowire::{
    zyanyad_request, zyanyad_response, NotifyBlockAddedRequestMessage, NotifyFinalityConflictRequestMessage,
    NotifyNewBlockTemplateRequestMessage, NotifyPruningPointUtxoSetOverrideRequestMessage, NotifySinkBlueScoreChangedRequestMessage,
    NotifyUtxosChangedRequestMessage, NotifyVirtualChainChangedRequestMessage, NotifyVirtualDaaScoreChangedRequestMessage,
    ZyanyadRequest, ZyanyadResponse,
};

impl ZyanyadRequest {
    pub fn from_notification_type(scope: &Scope, command: Command) -> Self {
        ZyanyadRequest { id: 0, payload: Some(zyanyad_request::Payload::from_notification_type(scope, command)) }
    }

    pub fn is_subscription(&self) -> bool {
        self.payload.as_ref().is_some_and(|x| x.is_subscription())
    }
}

impl zyanyad_request::Payload {
    pub fn from_notification_type(scope: &Scope, command: Command) -> Self {
        match scope {
            Scope::BlockAdded(_) => {
                zyanyad_request::Payload::NotifyBlockAddedRequest(NotifyBlockAddedRequestMessage { command: command.into() })
            }
            Scope::NewBlockTemplate(_) => {
                zyanyad_request::Payload::NotifyNewBlockTemplateRequest(NotifyNewBlockTemplateRequestMessage {
                    command: command.into(),
                })
            }

            Scope::VirtualChainChanged(ref scope) => {
                zyanyad_request::Payload::NotifyVirtualChainChangedRequest(NotifyVirtualChainChangedRequestMessage {
                    command: command.into(),
                    include_accepted_transaction_ids: scope.include_accepted_transaction_ids,
                })
            }
            Scope::FinalityConflict(_) => {
                zyanyad_request::Payload::NotifyFinalityConflictRequest(NotifyFinalityConflictRequestMessage {
                    command: command.into(),
                })
            }
            Scope::FinalityConflictResolved(_) => {
                zyanyad_request::Payload::NotifyFinalityConflictRequest(NotifyFinalityConflictRequestMessage {
                    command: command.into(),
                })
            }
            Scope::UtxosChanged(ref scope) => zyanyad_request::Payload::NotifyUtxosChangedRequest(NotifyUtxosChangedRequestMessage {
                addresses: scope.addresses.iter().map(|x| x.into()).collect::<Vec<String>>(),
                command: command.into(),
            }),
            Scope::SinkBlueScoreChanged(_) => {
                zyanyad_request::Payload::NotifySinkBlueScoreChangedRequest(NotifySinkBlueScoreChangedRequestMessage {
                    command: command.into(),
                })
            }
            Scope::VirtualDaaScoreChanged(_) => {
                zyanyad_request::Payload::NotifyVirtualDaaScoreChangedRequest(NotifyVirtualDaaScoreChangedRequestMessage {
                    command: command.into(),
                })
            }
            Scope::PruningPointUtxoSetOverride(_) => {
                zyanyad_request::Payload::NotifyPruningPointUtxoSetOverrideRequest(NotifyPruningPointUtxoSetOverrideRequestMessage {
                    command: command.into(),
                })
            }
        }
    }

    pub fn is_subscription(&self) -> bool {
        use crate::protowire::zyanyad_request::Payload;
        matches!(
            self,
            Payload::NotifyBlockAddedRequest(_)
                | Payload::NotifyVirtualChainChangedRequest(_)
                | Payload::NotifyFinalityConflictRequest(_)
                | Payload::NotifyUtxosChangedRequest(_)
                | Payload::NotifySinkBlueScoreChangedRequest(_)
                | Payload::NotifyVirtualDaaScoreChangedRequest(_)
                | Payload::NotifyPruningPointUtxoSetOverrideRequest(_)
                | Payload::NotifyNewBlockTemplateRequest(_)
                | Payload::StopNotifyingUtxosChangedRequest(_)
                | Payload::StopNotifyingPruningPointUtxoSetOverrideRequest(_)
        )
    }
}

impl ZyanyadResponse {
    pub fn is_notification(&self) -> bool {
        match self.payload {
            Some(ref payload) => payload.is_notification(),
            None => false,
        }
    }
}

#[allow(clippy::match_like_matches_macro)]
impl zyanyad_response::Payload {
    pub fn is_notification(&self) -> bool {
        use crate::protowire::zyanyad_response::Payload;
        match self {
            Payload::BlockAddedNotification(_) => true,
            Payload::VirtualChainChangedNotification(_) => true,
            Payload::FinalityConflictNotification(_) => true,
            Payload::FinalityConflictResolvedNotification(_) => true,
            Payload::UtxosChangedNotification(_) => true,
            Payload::SinkBlueScoreChangedNotification(_) => true,
            Payload::VirtualDaaScoreChangedNotification(_) => true,
            Payload::PruningPointUtxoSetOverrideNotification(_) => true,
            Payload::NewBlockTemplateNotification(_) => true,
            _ => false,
        }
    }
}
