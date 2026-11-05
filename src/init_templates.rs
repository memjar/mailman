use clap::{Parser, ValueEnum};
use pimalaya_cli::printer::Printer;

/// Provider presets `init` knows the server details for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// iCloud / me.com (app password, IMAP 993 + SMTP 587 STARTTLS).
    Icloud,
    /// Gmail (app password, IMAP 993 + SMTPS 465).
    Gmail,
    /// Any other IMAP/SMTP server; hosts given with --imap-server/--smtp-server.
    Imap,
}

/// Print a ready-to-paste account block for a config file.
#[derive(Debug, Parser)]
pub struct InitCommand {
    #[command(flatten)]
    pub provider: ProviderArg,
    #[command(flatten)]
    pub identity: IdentityArg,
    #[command(flatten)]
    pub account_id: AccountIdArg,
}

#[derive(Debug, clap::Args)]
pub struct ProviderArg {
    /// Which provider to template.
    #[arg(short, long, value_enum, default_value_t = Provider::Imap)]
    pub provider: Provider,
    /// IMAP server URL (e.g. imaps://imap.example.com:993); required for --provider imap.
    #[arg(long, value_name = "URL")]
    pub imap_server: Option<String>,
    /// SMTP server URL (e.g. smtp://smtp.example.com:587); required for --provider imap.
    #[arg(long, value_name = "URL")]
    pub smtp_server: Option<String>,
    /// Whether the SMTP server needs STARTTLS; default true for smtp://, false for smtps://.
    #[arg(long)]
    pub starttls: Option<bool>,
}

#[derive(Debug, clap::Args)]
pub struct IdentityArg {
    /// Email address of the account.
    #[arg(short, long, value_name = "EMAIL")]
    pub email: String,
    /// Display name.
    #[arg(long, value_name = "NAME", default_value = "Mailman User")]
    pub name: String,
}

#[derive(Debug, clap::Args)]
pub struct AccountIdArg {
    /// Key the account block is written under; defaults to the local part of --email.
    #[arg(long, value_name = "ID")]
    pub account_id: Option<String>,
}

/// The account block and its id, built from the command's arguments.
///
/// `execute` prints what this returns; the tests assert on it, so the shape
/// clients are handed has one source.
pub fn template_of(cmd: &InitCommand) -> anyhow::Result<(String, String)> {
    let id = cmd
        .account_id
        .account_id
        .clone()
        .unwrap_or_else(|| local_part(&cmd.identity.email));
    let (imap_server, smtp_server, sent_alias, starttls) =
        match cmd.provider.provider {
            Provider::Icloud => (
                "imaps://imap.mail.me.com:993",
                "smtp://smtp.mail.me.com:587",
                Some("Sent Messages"),
                true,
            ),
            Provider::Gmail => (
                "imaps://imap.gmail.com:993",
                "smtps://smtp.gmail.com:465",
                None,
                false,
            ),
            Provider::Imap => {
                let imap =
                    cmd.provider.imap_server.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("--provider imap needs --imap-server <URL>")
                    })?;
                let smtp =
                    cmd.provider.smtp_server.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("--provider imap needs --smtp-server <URL>")
                    })?;
                let starttls = cmd
                    .provider
                    .starttls
                    .unwrap_or_else(|| smtp.starts_with("smtp://"));
                (imap, smtp, None, starttls)
            }
        };
    // iCloud signs IMAP in with the local part, SMTP with the full address.
    let imap_login = match cmd.provider.provider {
        Provider::Icloud => local_part(&cmd.identity.email),
        _ => cmd.identity.email.clone(),
    };
    let pw_cmd = format!("security find-generic-password -s mailman -a {id} -w");

    let mut out = format!("[accounts.{id}]\n");
    out.push_str(&format!("name = {:?}\n", cmd.identity.name));
    out.push_str(&format!("email = {:?}\n", cmd.identity.email));
    out.push_str("default = true\n\n");
    out.push_str(&format!("imap.server = {:?}\n", imap_server));
    out.push_str(&format!("imap.sasl.plain.username = {:?}\n", imap_login));
    out.push_str(&format!(
        "imap.sasl.plain.password.command = {:?}\n\n",
        pw_cmd
    ));
    out.push_str(&format!("smtp.server = {:?}\n", smtp_server));
    if starttls {
        out.push_str("smtp.starttls = true\n");
    }
    out.push_str(&format!(
        "smtp.sasl.plain.username = {:?}\n",
        cmd.identity.email
    ));
    out.push_str(&format!(
        "smtp.sasl.plain.password.command = {:?}\n",
        pw_cmd
    ));
    if let Some(sent) = sent_alias {
        out.push_str(&format!("\nmailbox.alias.sent = {:?}\n", sent));
    }
    out.push_str(&format!(
        "\n# 1. store the secret:  security add-generic-password -s mailman -a {id} -w\n\
         #    (iCloud and Gmail need an app password from the provider's account page)\n\
         # 2. paste this block into ~/.config/mailman/config.toml\n\
         # 3. verify:  mailman account check\n"
    ));
    Ok((id, out))
}

fn local_part(email: &str) -> String {
    email.split('@').next().unwrap_or(email).to_string()
}

impl InitCommand {
    pub fn execute(self, printer: &mut impl Printer) -> anyhow::Result<()> {
        let (id, block) = template_of(&self)?;
        if printer.is_json() {
            printer.out(serde_json::json!({
                "account": id,
                "provider": self.provider.provider,
                "config_toml": block,
            }))?;
        } else {
            printer.out(block)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(args: &[&str]) -> String {
        let cmd = InitCommand::try_parse_from(args).unwrap();
        template_of(&cmd).unwrap().1
    }

    #[test]
    fn icloud_block_matches_the_live_verified_shape() {
        let out = block(&["init", "--provider", "icloud", "--email", "sam@icloud.com"]);
        assert!(out.contains("[accounts.sam]"));
        assert!(out.contains("imaps://imap.mail.me.com:993"));
        assert!(out.contains("smtp://smtp.mail.me.com:587"));
        assert!(out.contains("smtp.starttls = true"));
        assert!(out.contains("imap.sasl.plain.username = \"sam\""));
        assert!(out.contains("smtp.sasl.plain.username = \"sam@icloud.com\""));
        assert!(out.contains("mailbox.alias.sent = \"Sent Messages\""));
        assert!(out.contains("security find-generic-password -s mailman -a sam -w"));
    }

    #[test]
    fn gmail_block_uses_smtps_and_the_full_login() {
        let out = block(&["init", "--provider", "gmail", "--email", "sam@gmail.com"]);
        assert!(out.contains("smtps://smtp.gmail.com:465"));
        assert!(!out.contains("smtp.starttls = true"));
        assert!(out.contains("imap.sasl.plain.username = \"sam@gmail.com\""));
    }

    #[test]
    fn imap_provider_needs_both_servers() {
        let cmd = InitCommand::try_parse_from(["init", "--provider", "imap", "--email", "a@b.c"])
            .unwrap();
        let err = template_of(&cmd).unwrap_err().to_string();
        assert!(err.contains("--imap-server"), "unexpected: {err}");

        let out = block(&[
            "init",
            "--provider",
            "imap",
            "--email",
            "me@corp.example",
            "--imap-server",
            "imaps://mail.corp.example:993",
            "--smtp-server",
            "smtp://mail.corp.example:587",
        ]);
        assert!(out.contains("imaps://mail.corp.example:993"));
        assert!(out.contains("smtp.starttls = true"));
    }

    #[test]
    fn account_id_overrides_the_local_part() {
        let cmd = InitCommand::try_parse_from([
            "init",
            "--provider",
            "icloud",
            "--email",
            "sam@icloud.com",
            "--account-id",
            "client-a",
        ])
        .unwrap();
        let (id, out) = template_of(&cmd).unwrap();
        assert_eq!(id, "client-a");
        assert!(out.contains("[accounts.client-a]"));
        assert!(out.contains("-a client-a -w"));
    }
}
