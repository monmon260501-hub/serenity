use serde::Serialize;

#[cfg(feature = "model")]
use crate::builder::{
    Builder,
    CreateInteractionResponse,
    CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
    EditInteractionResponse,
};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::prelude::*;
use crate::json::from_value;
use crate::model::prelude::*;
use crate::model::utils::deserialize_val;

/// An interaction triggered by a modal submit.
///
/// [Discord docs](https://docs.discord.com/developers/interactions/receiving-and-responding#interaction-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct ModalInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: ModalInteractionData,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// Channel that the interaction was sent from.
    pub channel: Option<PartialChannel>,
    /// The channel Id this interaction was sent from.
    pub channel_id: ChannelId,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    /// The `user` object for the invoking user.
    #[serde(default)]
    pub user: User,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
    /// The message this interaction was triggered by
    ///
    /// **Note**: Does not exist if the modal interaction originates from an application command
    /// interaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Box<Message>>,
    /// Permissions the app or bot has within the channel the interaction was sent from.
    pub app_permissions: Option<Permissions>,
    /// The selected language of the invoking user.
    pub locale: String,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
    /// For monetized applications, any entitlements of the invoking user.
    pub entitlements: Vec<Entitlement>,
    /// Attachment size limit in bytes.
    pub attachment_size_limit: u32,
}

#[cfg(feature = "model")]
impl ModalInteraction {
    /// Gets the interaction response.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if there is no interaction response.
    pub async fn get_response(&self, http: impl AsRef<Http>) -> Result<Message> {
        http.as_ref().get_original_interaction_response(&self.token).await
    }

    /// Creates a response to the interaction received.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    pub async fn create_response(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponse,
    ) -> Result<()> {
        builder.execute(cache_http, (self.id, &self.token)).await
    }

    /// Edits the initial interaction response.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is an error in
    /// deserializing the API response.
    pub async fn edit_response(
        &self,
        cache_http: impl CacheHttp,
        builder: EditInteractionResponse,
    ) -> Result<Message> {
        builder.execute(cache_http, &self.token).await
    }

    /// Deletes the initial interaction response.
    ///
    /// Does not work on ephemeral messages.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was already
    /// deleted.
    pub async fn delete_response(&self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_original_interaction_response(&self.token).await
    }

    /// Creates a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    pub async fn create_followup(
        &self,
        cache_http: impl CacheHttp,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder.execute(cache_http, (None, &self.token)).await
    }

    /// Edits a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    pub async fn edit_followup(
        &self,
        cache_http: impl CacheHttp,
        message_id: impl Into<MessageId>,
        builder: CreateInteractionResponseFollowup,
    ) -> Result<Message> {
        builder.execute(cache_http, (Some(message_id.into()), &self.token)).await
    }

    /// Deletes a followup message.
    ///
    /// # Errors
    ///
    /// May return [`Error::Http`] if the API returns an error. Such as if the response was already
    /// deleted.
    pub async fn delete_followup<M: Into<MessageId>>(
        &self,
        http: impl AsRef<Http>,
        message_id: M,
    ) -> Result<()> {
        http.as_ref().delete_followup_message(&self.token, message_id.into()).await
    }

    /// Helper function to defer an interaction.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error, or an [`Error::Json`] if there is
    /// an error in deserializing the API response.
    pub async fn defer(&self, cache_http: impl CacheHttp) -> Result<()> {
        self.create_response(cache_http, CreateInteractionResponse::Acknowledge).await
    }

    /// Helper function to defer an interaction ephemerally
    ///
    /// # Errors
    ///
    /// May also return an [`Error::Http`] if the API returns an error, or an [`Error::Json`] if
    /// there is an error in deserializing the API response.
    pub async fn defer_ephemeral(&self, cache_http: impl CacheHttp) -> Result<()> {
        let builder = CreateInteractionResponse::Defer(
            CreateInteractionResponseMessage::new().ephemeral(true),
        );
        self.create_response(cache_http, builder).await
    }
}

// Manual impl needed to insert guild_id into resolved Role's
impl<'de> Deserialize<'de> for ModalInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut interaction = Self::deserialize(deserializer)?; // calls #[serde(remote)]-generated inherent method
        if let (Some(guild_id), Some(member)) = (interaction.guild_id, &mut interaction.member) {
            member.guild_id = guild_id;
            // If `member` is present, `user` wasn't sent and is still filled with default data
            interaction.user = member.user.clone();
        }
        Ok(interaction)
    }
}

impl Serialize for ModalInteraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        Self::serialize(self, serializer) // calls #[serde(remote)]-generated inherent method
    }
}

/// A modal submit interaction data, provided by [`ModalInteraction::data`]
///
/// [Discord docs](https://docs.discord.com/developers/interactions/receiving-and-responding#interaction-object-modal-submit-data-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ModalInteractionData {
    /// The custom id of the modal
    pub custom_id: String,
    /// Legacy action row components.
    ///
    /// This remains populated for existing text-input modals for backwards compatibility.
    pub components: Vec<ActionRow>,
    /// All top-level components in the submitted modal.
    pub modal_components: Vec<ModalComponent>,
    /// Resolved entities referenced by modal inputs, including uploaded attachments.
    pub resolved: CommandDataResolved,
}

impl<'de> Deserialize<'de> for ModalInteractionData {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            custom_id: String,
            components: Vec<ModalComponent>,
            #[serde(default)]
            resolved: CommandDataResolved,
        }

        let raw = Raw::deserialize(deserializer)?;
        let components = raw
            .components
            .iter()
            .filter_map(|component| match component {
                ModalComponent::ActionRow(row) => Some(row.clone()),
                ModalComponent::Label(_) => None,
            })
            .collect();
        Ok(Self {
            custom_id: raw.custom_id,
            components,
            modal_components: raw.components,
            resolved: raw.resolved,
        })
    }
}

impl Serialize for ModalInteractionData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Raw<'a> {
            custom_id: &'a str,
            components: Vec<ModalComponent>,
            #[serde(skip_serializing_if = "resolved_is_empty")]
            resolved: CommandDataResolved,
        }

        let components = if self.modal_components.is_empty() {
            self.components.iter().cloned().map(ModalComponent::ActionRow).collect()
        } else {
            self.modal_components.clone()
        };
        Raw {
            custom_id: &self.custom_id,
            components,
            resolved: self.resolved.clone(),
        }
        .serialize(serializer)
    }
}

fn resolved_is_empty(resolved: &CommandDataResolved) -> bool {
    resolved.users.is_empty()
        && resolved.members.is_empty()
        && resolved.roles.is_empty()
        && resolved.channels.is_empty()
        && resolved.messages.is_empty()
        && resolved.attachments.is_empty()
}

/// A top-level component in a modal submission.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ModalComponent {
    ActionRow(ActionRow),
    Label(Label),
}

impl<'de> Deserialize<'de> for ModalComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let raw_kind =
            map.get("type").ok_or_else(|| serde::de::Error::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::ActionRow => from_value(value).map(Self::ActionRow),
            ComponentType::Label => from_value(value).map(Self::Label),
            ComponentType::Unknown(i) => {
                return Err(serde::de::Error::custom(format_args!(
                    "Unknown modal component type {i}"
                )))
            },
            kind => {
                return Err(serde::de::Error::custom(format_args!(
                    "Invalid modal component {kind:?}"
                )))
            },
        }
        .map_err(serde::de::Error::custom)
    }
}

impl Serialize for ModalComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        match self {
            Self::ActionRow(component) => component.serialize(serializer),
            Self::Label(component) => component.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{from_value, json};

    #[test]
    fn deserializes_legacy_action_row_without_breaking_existing_access() {
        let data: ModalInteractionData = from_value(json!({
            "custom_id": "legacy",
            "components": [{
                "type": 1,
                "components": [{
                    "type": 4,
                    "custom_id": "name",
                    "value": "Serenity"
                }]
            }]
        }))
        .unwrap();

        assert_eq!(data.components.len(), 1);
        assert_eq!(data.modal_components.len(), 1);
        let ActionRowComponent::InputText(input) = &data.components[0].components[0] else {
            panic!("expected legacy input text");
        };
        assert_eq!(input.value.as_deref(), Some("Serenity"));
    }

    #[test]
    fn deserializes_file_upload_and_resolved_attachment() {
        let data: ModalInteractionData = from_value(json!({
            "custom_id": "upload",
            "components": [{
                "type": 18,
                "id": 1,
                "component": {
                    "type": 19,
                    "id": 2,
                    "custom_id": "image",
                    "values": ["42"]
                }
            }],
            "resolved": {
                "attachments": {
                    "42": {
                        "id": "42",
                        "filename": "image.png",
                        "proxy_url": "https://cdn.example/image.png",
                        "size": 128,
                        "url": "https://cdn.example/image.png",
                        "content_type": "image/png"
                    }
                }
            }
        }))
        .unwrap();

        assert!(data.components.is_empty());
        let ModalComponent::Label(label) = &data.modal_components[0] else {
            panic!("expected label");
        };
        let LabelComponent::FileUpload(upload) = &label.component else {
            panic!("expected file upload");
        };
        assert_eq!(upload.custom_id, "image");
        assert_eq!(upload.values, [AttachmentId::new(42)]);
        assert_eq!(data.resolved.attachments[&AttachmentId::new(42)].filename, "image.png");
    }
}
