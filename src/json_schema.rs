//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `mailman-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry.
//!
//! Protocol-specific entries are gated behind the same cargo features as
//! their command modules, so the registry stays coherent under any feature
//! combination, none included.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::JsonSchemaCommand

use pimalaya_cli::{
    clap::commands::JsonSchemaRegistry,
    error::ErrorReport,
    output::{Bytes, Paginated},
};
use schemars::JsonSchema;

use crate::error::ErrorCode;
#[cfg(any(
    feature = "gmail",
    feature = "msgraph",
    feature = "jmap",
    feature = "maildir",
    feature = "mbox",
    feature = "m2dir",
    feature = "sieve"
))]
use crate::shared::receipt::{IdReceipt, IdsReceipt};

/// Builds the command-to-schema registry consumed by `json-schema <DIR>`.
///
/// Each value describes the type the command hands to the printer, and
/// `mailman-error` the failure object every command prints instead.
/// Paginated listings use the `Paginated<T>` wrapper, so the extra
/// `nextPage` field is described too, and a command printing one of two
/// types is described by both, through `OneOf`.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("mailman", env!("CARGO_PKG_VERSION"));

    macro_rules! insert {
        ($key:expr, $ty:ty) => {
            registry.insert::<$ty>($key);
        };
    }

    #[cfg(feature = "imap")]
    macro_rules! imap {
        ($key:expr, $ty:ty) => {
            registry.insert::<crate::imap::outcome::ImapOutcome<$ty>>($key);
        };
    }

    #[cfg(any(backend, feature = "smtp"))]
    {
        insert!(
            "mailman-mailbox-list",
            crate::shared::mailbox::list::MailboxListOutput
        );
        insert!(
            "mailman-mailbox-create",
            crate::shared::mailbox::create::MailboxCreateOutput
        );
        insert!(
            "mailman-mailbox-rename",
            crate::shared::mailbox::rename::MailboxRenameOutput
        );
        insert!(
            "mailman-mailbox-delete",
            crate::shared::mailbox::delete::MailboxDeleteOutput
        );
        insert!(
            "mailman-envelope-list",
            Paginated<crate::shared::envelope::list::EnvelopeListOutput>
        );
        // NOTE: `envelope search` renders the page `list` renders.
        insert!(
            "mailman-envelope-search",
            Paginated<crate::shared::envelope::list::EnvelopeListOutput>
        );
        insert!("mailman-flag-add", crate::shared::flag::add::FlagAddOutput);
        insert!("mailman-flag-set", crate::shared::flag::set::FlagSetOutput);
        insert!(
            "mailman-flag-remove",
            crate::shared::flag::remove::FlagRemoveOutput
        );
        insert!(
            "mailman-message-add",
            crate::shared::message::add::MessageAddOutput
        );
        insert!(
            "mailman-message-copy",
            crate::shared::message::copy::MessageCopyOutput
        );
        insert!(
            "mailman-message-move",
            crate::shared::message::mv::MessageMoveOutput
        );
        insert!(
            "mailman-message-send",
            crate::shared::message::handler::MessageRouteOutput
        );
        // NOTE: `message read --raw` prints the message as bytes.
        insert!(
            "mailman-message-read",
            OneOf<crate::shared::message::read::MessageReadOutput, Bytes>
        );
        // NOTE: `message parse` prints the `message read` view, or under
        // `--part` the part `attachment download --stdout` prints.
        insert!(
            "mailman-message-parse",
            crate::shared::message::parse::MessageParseOutput
        );
        insert!(
            "mailman-message-delete",
            crate::shared::message::delete::MessageDeleteOutput
        );
        // NOTE: the composers print the template when neither saving nor
        // sending, and the receipt of the save or the send otherwise.
        insert!(
            "mailman-message-compose",
            OneOf<
                crate::shared::message::handler::MessageTemplateOutput,
                crate::shared::message::handler::MessageRouteOutput,
            >
        );
        insert!(
            "mailman-message-reply",
            OneOf<
                crate::shared::message::handler::MessageTemplateOutput,
                crate::shared::message::handler::MessageRouteOutput,
            >
        );
        insert!(
            "mailman-message-forward",
            OneOf<
                crate::shared::message::handler::MessageTemplateOutput,
                crate::shared::message::handler::MessageRouteOutput,
            >
        );
        insert!(
            "mailman-attachment-list",
            crate::shared::attachment::list::AttachmentListOutput
        );
        // NOTE: `attachment download` reports the `AttachmentListOutput` of
        // `list`, or the one part `--stdout --json` reads.
        insert!(
            "mailman-attachment-download",
            crate::shared::attachment::download::AttachmentDownloadOutput
        );
        insert!(
            "mailman-vacation-get",
            crate::shared::vacation::get::VacationGetOutput
        );
        insert!(
            "mailman-vacation-set",
            crate::shared::vacation::set::VacationSetOutput
        );
    }

    #[cfg(feature = "wizard")]
    insert!(
        "mailman-configure",
        crate::wizard::configure::ConfigureOutput
    );
    insert!(
        "mailman-account-list",
        crate::account::list::AccountListOutput
    );
    insert!(
        "mailman-account-check",
        crate::account::check::AccountCheckOutput
    );
    insert!("mailman-init", crate::init_templates::InitOutput);
    insert!(
        "mailman-account-capability",
        crate::account::capability::AccountCapabilityOutput
    );
    #[cfg(feature = "imap")]
    {
        imap!("mailman-imap-id", crate::imap::id::ImapIdOutput);
        imap!(
            "mailman-imap-status",
            crate::imap::mailbox::status::ImapMailboxStatusOutput
        );
        imap!(
            "mailman-imap-list",
            crate::imap::mailbox::list::ImapMailboxListOutput
        );
        imap!(
            "mailman-imap-search",
            crate::imap::envelope::search::ImapSearchOutput
        );
        imap!(
            "mailman-imap-sort",
            crate::imap::envelope::sort::ImapSortOutput
        );
        imap!(
            "mailman-imap-thread",
            crate::imap::envelope::thread::ImapThreadOutput
        );
        imap!(
            "mailman-imap-flag",
            crate::imap::flag::list::ImapFlagListOutput<'static>
        );
        imap!("mailman-imap-fetch", crate::imap::fetch::ImapFetchOutput);
    }

    #[cfg(feature = "sieve")]
    {
        insert!(
            "mailman-sieve-capability",
            crate::sieve::capability::SieveCapabilityOutput
        );
        insert!("mailman-sieve-list", crate::sieve::list::SieveListOutput);
        insert!("mailman-sieve-get", crate::sieve::get::SieveScriptOutput);
    }

    #[cfg(feature = "jmap")]
    {
        insert!("mailman-jmap-query", crate::jmap::query::JmapQueryOutput);
        insert!(
            "mailman-jmap-mailbox-get",
            crate::jmap::mailbox::query::JmapMailboxQueryOutput
        );
        insert!(
            "mailman-jmap-mailbox-query",
            crate::jmap::mailbox::query::JmapMailboxQueryOutput
        );
        insert!(
            "mailman-jmap-email-get",
            crate::jmap::email::query::JmapEmailQueryOutput
        );
        insert!(
            "mailman-jmap-email-query",
            crate::jmap::email::query::JmapEmailQueryOutput
        );
        insert!(
            "mailman-jmap-email-parse",
            crate::jmap::email::parse::JmapEmailParseOutput
        );
        insert!(
            "mailman-jmap-thread-get",
            crate::jmap::thread::get::JmapThreadGetOutput
        );
        insert!(
            "mailman-jmap-identity-get",
            crate::jmap::identity::get::JmapIdentityGetOutput
        );
        insert!(
            "mailman-jmap-submission-get",
            crate::jmap::submission::query::JmapSubmissionQueryOutput
        );
        insert!(
            "mailman-jmap-submission-query",
            crate::jmap::submission::query::JmapSubmissionQueryOutput
        );
        insert!(
            "mailman-jmap-submission-create",
            crate::jmap::submission::query::JmapSubmissionQueryOutput
        );
        insert!(
            "mailman-jmap-vacation-response-get",
            crate::jmap::vacation::get::JmapVacationResponseGetOutput
        );
    }

    #[cfg(feature = "gmail")]
    {
        insert!(
            "mailman-gmail-profile-get",
            crate::gmail::profile::get::GmailProfileOutput
        );
        insert!(
            "mailman-gmail-labels-list",
            crate::gmail::labels::list::GmailLabelListOutput
        );
        insert!(
            "mailman-gmail-labels-get",
            crate::gmail::labels::list::GmailLabelListOutput
        );
        insert!(
            "mailman-gmail-messages-list",
            Paginated<crate::gmail::messages::list::GmailMessageListOutput>
        );
        insert!(
            "mailman-gmail-messages-get",
            OneOf<crate::gmail::messages::get::GmailMessageGetOutput, Bytes>
        );
        insert!(
            "mailman-gmail-drafts-list",
            Paginated<crate::gmail::drafts::list::GmailDraftListOutput>
        );
        insert!(
            "mailman-gmail-drafts-get",
            OneOf<crate::gmail::drafts::get::GmailDraftGetOutput, Bytes>
        );
        insert!(
            "mailman-gmail-drafts-create",
            crate::gmail::drafts::GmailDraftWriteOutput
        );
        insert!(
            "mailman-gmail-drafts-update",
            crate::gmail::drafts::GmailDraftWriteOutput
        );
        insert!(
            "mailman-gmail-threads-list",
            Paginated<crate::gmail::threads::list::GmailThreadListOutput>
        );
        insert!(
            "mailman-gmail-threads-get",
            crate::gmail::threads::get::GmailThreadGetOutput
        );
        insert!(
            "mailman-gmail-history-list",
            Paginated<crate::gmail::history::list::GmailHistoryListOutput>
        );
        insert!(
            "mailman-gmail-settings-filters-list",
            crate::gmail::settings::filters::list::GmailSettingsFilterListOutput
        );
        insert!(
            "mailman-gmail-settings-filters-get",
            crate::gmail::settings::filters::get::GmailSettingsFilterGetOutput
        );
        insert!(
            "mailman-gmail-settings-forwarding-addresses-list",
            crate::gmail::settings::forwarding_addresses::list::GmailSettingsForwardingAddressListOutput
        );
        insert!(
            "mailman-gmail-settings-forwarding-addresses-get",
            crate::gmail::settings::forwarding_addresses::get::GmailSettingsForwardingAddressGetOutput
        );
        insert!(
            "mailman-gmail-settings-delegates-list",
            crate::gmail::settings::delegates::list::GmailSettingsDelegateListOutput
        );
        insert!(
            "mailman-gmail-settings-delegates-get",
            crate::gmail::settings::delegates::get::GmailSettingsDelegateGetOutput
        );
        insert!(
            "mailman-gmail-settings-send-as-list",
            crate::gmail::settings::sendas::list::GmailSettingsSendAsListOutput
        );
        insert!(
            "mailman-gmail-settings-send-as-get",
            crate::gmail::settings::sendas::get::GmailSettingsSendAsGetOutput
        );
        insert!(
            "mailman-gmail-settings-vacation-get",
            crate::gmail::settings::vacation::get::GmailSettingsVacationGetOutput
        );
        insert!(
            "mailman-gmail-settings-auto-forwarding-get",
            crate::gmail::settings::autoforwarding::get::GmailSettingsAutoForwardingGetOutput
        );
        insert!(
            "mailman-gmail-settings-pop-get",
            crate::gmail::settings::pop::get::GmailSettingsPopGetOutput
        );
        insert!(
            "mailman-gmail-settings-imap-get",
            crate::gmail::settings::imap::get::GmailSettingsImapGetOutput
        );
        insert!(
            "mailman-gmail-settings-language-get",
            crate::gmail::settings::language::get::GmailSettingsLanguageGetOutput
        );
    }

    #[cfg(feature = "msgraph")]
    {
        insert!(
            "mailman-msgraph-profile-get",
            crate::msgraph::profile::get::MsgraphProfileOutput
        );
        insert!(
            "mailman-msgraph-mail-folders-list",
            crate::msgraph::mail_folders::list::MsgraphMailFolderListOutput
        );
        insert!(
            "mailman-msgraph-mail-folders-child-folders",
            crate::msgraph::mail_folders::list::MsgraphMailFolderListOutput
        );
        insert!(
            "mailman-msgraph-mail-folders-get",
            crate::msgraph::mail_folders::list::MsgraphMailFolderListOutput
        );
        insert!(
            "mailman-msgraph-messages-list",
            Paginated<crate::msgraph::messages::list::MsgraphMessageListOutput>
        );
        insert!(
            "mailman-msgraph-messages-get",
            OneOf<crate::msgraph::messages::get::MsgraphMessageGetOutput, Bytes>
        );
        insert!(
            "mailman-msgraph-attachments-list",
            crate::msgraph::attachments::list::MsgraphAttachmentListOutput
        );
    }

    #[cfg(feature = "maildir")]
    {
        insert!(
            "mailman-maildir-list",
            crate::maildir::list::MaildirListOutput
        );
        insert!(
            "mailman-maildir-flag-list",
            crate::maildir::flag::list::MaildirFlagListOutput
        );
        insert!(
            "mailman-maildir-message-save",
            crate::maildir::message::save::MaildirMessageSaveOutput
        );
    }

    #[cfg(feature = "mbox")]
    {
        insert!("mailman-mbox-list", crate::mbox::list::MboxListOutput);
        insert!(
            "mailman-mbox-flag-list",
            crate::mbox::flag::list::MboxFlagListOutput
        );
        insert!(
            "mailman-mbox-message-save",
            crate::mbox::message::save::MboxMessageSaveOutput
        );
    }

    #[cfg(feature = "m2dir")]
    {
        insert!("mailman-m2dir-list", crate::m2dir::list::M2dirListOutput);
        insert!(
            "mailman-m2dir-flag-list",
            crate::m2dir::flag::list::M2dirFlagListOutput
        );
        insert!(
            "mailman-m2dir-message-save",
            crate::m2dir::message::save::M2dirMessageSaveOutput
        );
    }

    #[cfg(feature = "pimdir")]
    {
        insert!(
            "mailman-pimdir-queue-list",
            crate::pimdir::queue::list::PimdirQueueListOutput
        );
        insert!(
            "mailman-pimdir-queue-cancel",
            crate::pimdir::queue::cancel::PimdirQueueCancelOutput
        );
        insert!(
            "mailman-pimdir-queue-show",
            crate::pimdir::queue::show::PimdirQueueShowOutput
        );
        insert!(
            "mailman-pimdir-message-add",
            crate::pimdir::message::add::PimdirMessageAddOutput
        );
        insert!(
            "mailman-pimdir-message-send",
            crate::pimdir::message::send::PimdirMessageSendOutput
        );
        insert!(
            "mailman-pimdir-flag-list",
            crate::pimdir::flag::list::PimdirFlagListOutput
        );
        insert!(
            "mailman-pimdir-flag-add",
            crate::pimdir::flag::PimdirFlagStageOutput
        );
        insert!(
            "mailman-pimdir-flag-set",
            crate::pimdir::flag::PimdirFlagStageOutput
        );
        insert!(
            "mailman-pimdir-flag-remove",
            crate::pimdir::flag::PimdirFlagStageOutput
        );
        insert!(
            "mailman-pimdir-mailbox-create",
            crate::pimdir::mailbox::create::PimdirMailboxCreateOutput
        );
        insert!(
            "mailman-pimdir-mailbox-rename",
            crate::pimdir::mailbox::rename::PimdirMailboxRenameOutput
        );
        insert!(
            "mailman-pimdir-mailbox-delete",
            crate::pimdir::mailbox::delete::PimdirMailboxDeleteOutput
        );
        insert!(
            "mailman-pimdir-mailbox-list",
            crate::pimdir::mailbox::list::PimdirMailboxListOutput
        );
        insert!(
            "mailman-pimdir-attachment-list",
            crate::pimdir::attachment::list::PimdirAttachmentListOutput
        );
        // NOTE: `pimdir attachment search` prints the page `list` prints.
        insert!(
            "mailman-pimdir-attachment-search",
            crate::pimdir::attachment::list::PimdirAttachmentListOutput
        );
        // NOTE: `pimdir search --threads` prints the threads instead.
        insert!(
            "mailman-pimdir-search",
            OneOf<
                crate::pimdir::search::PimdirSearchOutput,
                crate::pimdir::search::PimdirSearchThreadsOutput,
            >
        );
        // NOTE: `pimdir performer` lists the candidates, or with a source
        // or `--clear` prints the choice.
        insert!(
            "mailman-pimdir-performer",
            OneOf<
                crate::pimdir::performer::PimdirPerformers,
                crate::pimdir::performer::PimdirPerformerChosen,
            >
        );
        insert!(
            "mailman-pimdir-count",
            crate::pimdir::count::PimdirCountOutput
        );
        insert!(
            "mailman-pimdir-thread",
            crate::pimdir::thread::PimdirThreadOutput
        );
        insert!("mailman-pimdir-tag", crate::pimdir::tag::PimdirTagOutput);
    }

    // NOTE: the writes of the protocol namespaces with no payload of their
    // own print a receipt naming what they acted on.

    #[cfg(feature = "imap")]
    {
        use crate::imap::outcome::{ImapDoneOutput, ImapMailboxOutput};

        imap!(
            "mailman-imap-select",
            crate::imap::mailbox::select::ImapMailboxSelectOutput
        );
        imap!("mailman-imap-create", ImapMailboxOutput);
        imap!("mailman-imap-delete", ImapMailboxOutput);
        imap!("mailman-imap-rename", ImapMailboxOutput);
        imap!("mailman-imap-subscribe", ImapMailboxOutput);
        imap!("mailman-imap-unsubscribe", ImapMailboxOutput);
        imap!("mailman-imap-close", ImapDoneOutput);
        imap!("mailman-imap-unselect", ImapDoneOutput);
        imap!(
            "mailman-imap-expunge",
            crate::imap::mailbox::expunge::ImapMailboxExpungeOutput
        );
        imap!("mailman-imap-store", crate::imap::fetch::ImapFetchOutput);
        imap!(
            "mailman-imap-append",
            crate::imap::message::save::ImapMessageAppendOutput
        );
        imap!(
            "mailman-imap-copy",
            crate::imap::message::copy::ImapMessageCopyOutput
        );
        imap!(
            "mailman-imap-move",
            crate::imap::message::copy::ImapMessageCopyOutput
        );
        insert!("mailman-imap-raw", crate::imap::raw::ImapRawOutput);
        registry.insert_events::<crate::imap::raw::ImapRawEvent>("mailman-imap-raw");
    }

    #[cfg(feature = "smtp")]
    {
        insert!("mailman-smtp-send", crate::smtp::send::SmtpSendOutput);
        insert!("mailman-smtp-raw", crate::smtp::raw::SmtpRawOutput);
    }

    #[cfg(feature = "sieve")]
    {
        insert!("mailman-sieve-activate", IdReceipt);
        insert!("mailman-sieve-delete", IdReceipt);
        insert!("mailman-sieve-rename", IdReceipt);
        insert!(
            "mailman-sieve-deactivate",
            crate::sieve::deactivate::SieveDeactivateOutput
        );
        insert!(
            "mailman-sieve-check",
            crate::sieve::check::SieveScriptOutput
        );
        insert!("mailman-sieve-put", crate::sieve::check::SieveScriptOutput);
        insert!("mailman-sieve-raw", crate::sieve::raw::SieveRawOutput);
        registry.insert_events::<crate::sieve::raw::SieveRawEvent>("mailman-sieve-raw");
    }

    #[cfg(feature = "jmap")]
    {
        insert!("mailman-jmap-email-copy", IdsReceipt);
        insert!("mailman-jmap-email-delete", IdsReceipt);
        insert!("mailman-jmap-email-update", IdsReceipt);
        insert!("mailman-jmap-email-export", Bytes);
        insert!(
            "mailman-jmap-email-import",
            crate::jmap::email::import::JmapEmailImportOutput
        );
        insert!(
            "mailman-jmap-email-read",
            crate::jmap::email::read::JmapEmailReadOutput
        );
        insert!("mailman-jmap-identity-create", IdReceipt);
        insert!("mailman-jmap-identity-update", IdReceipt);
        insert!("mailman-jmap-identity-delete", IdsReceipt);
        insert!("mailman-jmap-mailbox-create", IdReceipt);
        insert!("mailman-jmap-mailbox-update", IdReceipt);
        insert!("mailman-jmap-mailbox-destroy", IdsReceipt);
        insert!("mailman-jmap-submission-cancel", IdsReceipt);
        insert!("mailman-jmap-vacation-response-set", IdReceipt);
    }

    #[cfg(feature = "gmail")]
    {
        use crate::{gmail::settings, shared::output::SavedOutput};

        for key in [
            "mailman-gmail-drafts-delete",
            "mailman-gmail-drafts-send",
            "mailman-gmail-labels-create",
            "mailman-gmail-labels-delete",
            "mailman-gmail-labels-update",
            "mailman-gmail-messages-delete",
            "mailman-gmail-messages-import",
            "mailman-gmail-messages-insert",
            "mailman-gmail-messages-modify",
            "mailman-gmail-messages-send",
            "mailman-gmail-messages-trash",
            "mailman-gmail-messages-untrash",
            "mailman-gmail-settings-delegates-create",
            "mailman-gmail-settings-delegates-delete",
            "mailman-gmail-settings-filters-create",
            "mailman-gmail-settings-filters-delete",
            "mailman-gmail-settings-forwarding-addresses-create",
            "mailman-gmail-settings-forwarding-addresses-delete",
            "mailman-gmail-settings-send-as-create",
            "mailman-gmail-settings-send-as-delete",
            "mailman-gmail-settings-send-as-update",
            "mailman-gmail-settings-send-as-verify",
            "mailman-gmail-threads-delete",
            "mailman-gmail-threads-modify",
            "mailman-gmail-threads-trash",
            "mailman-gmail-threads-untrash",
        ] {
            insert!(key, IdReceipt);
        }

        insert!("mailman-gmail-messages-batch-delete", IdsReceipt);
        insert!("mailman-gmail-messages-batch-modify", IdsReceipt);
        insert!(
            "mailman-gmail-attachments-get",
            OneOf<Bytes, SavedOutput>
        );
        // NOTE: a settings write prints the settings the server holds
        // after it, as the matching `get` does.
        insert!(
            "mailman-gmail-settings-auto-forwarding-set",
            settings::autoforwarding::get::GmailSettingsAutoForwardingGetOutput
        );
        insert!(
            "mailman-gmail-settings-imap-set",
            settings::imap::get::GmailSettingsImapGetOutput
        );
        insert!(
            "mailman-gmail-settings-language-set",
            settings::language::get::GmailSettingsLanguageGetOutput
        );
        insert!(
            "mailman-gmail-settings-pop-set",
            settings::pop::get::GmailSettingsPopGetOutput
        );
        insert!(
            "mailman-gmail-settings-vacation-set",
            settings::vacation::get::GmailSettingsVacationGetOutput
        );
    }

    #[cfg(feature = "msgraph")]
    {
        use crate::shared::output::SavedOutput;

        for key in [
            "mailman-msgraph-attachments-create",
            "mailman-msgraph-attachments-delete",
            "mailman-msgraph-mail-folders-copy",
            "mailman-msgraph-mail-folders-create",
            "mailman-msgraph-mail-folders-delete",
            "mailman-msgraph-mail-folders-move",
            "mailman-msgraph-mail-folders-rename",
            "mailman-msgraph-messages-copy",
            "mailman-msgraph-messages-create",
            "mailman-msgraph-messages-delete",
            "mailman-msgraph-messages-move",
            "mailman-msgraph-messages-send-draft",
            "mailman-msgraph-messages-update",
        ] {
            insert!(key, IdReceipt);
        }

        insert!(
            "mailman-msgraph-messages-send",
            crate::msgraph::messages::send::MsgraphMessageSendOutput
        );
        insert!(
            "mailman-msgraph-attachments-get",
            OneOf<Bytes, SavedOutput>
        );
    }

    #[cfg(feature = "maildir")]
    {
        for key in [
            "mailman-maildir-create",
            "mailman-maildir-delete",
            "mailman-maildir-rename",
        ] {
            insert!(key, IdReceipt);
        }

        for key in [
            "mailman-maildir-flag-add",
            "mailman-maildir-flag-remove",
            "mailman-maildir-flag-set",
            "mailman-maildir-message-copy",
            "mailman-maildir-message-move",
        ] {
            insert!(key, IdsReceipt);
        }
    }

    #[cfg(feature = "mbox")]
    {
        for key in [
            "mailman-mbox-create",
            "mailman-mbox-delete",
            "mailman-mbox-rename",
        ] {
            insert!(key, IdReceipt);
        }

        for key in [
            "mailman-mbox-flag-add",
            "mailman-mbox-flag-remove",
            "mailman-mbox-flag-set",
            "mailman-mbox-message-copy",
            "mailman-mbox-message-move",
        ] {
            insert!(key, IdsReceipt);
        }
    }

    #[cfg(feature = "m2dir")]
    {
        for key in ["mailman-m2dir-create", "mailman-m2dir-delete"] {
            insert!(key, IdReceipt);
        }

        for key in [
            "mailman-m2dir-flag-add",
            "mailman-m2dir-flag-remove",
            "mailman-m2dir-flag-set",
        ] {
            insert!(key, IdsReceipt);
        }
    }

    insert!("mailman-error", ErrorReport<ErrorCode>);

    registry
}

/// The schema of a command printing one of two types, told apart by their
/// keys.
#[derive(JsonSchema)]
#[serde(untagged)]
#[allow(dead_code)]
enum OneOf<A, B> {
    A(A),
    B(B),
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::{cli::Cli, json_schema::registry};

    /// The commands printing nothing a schema could describe: the REPL
    /// and the MCP server speak their own protocols, and a bare
    /// `mailman` shows the help, or runs the wizard `configure`
    /// describes.
    const SILENT: &[&str] = &["mailman", "mailman-repl", "mailman-mcp"];

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<String>::new());
    }

    #[test]
    fn every_command_is_described() {
        let problems = registry().check(&Cli::command(), SILENT);
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }
}
