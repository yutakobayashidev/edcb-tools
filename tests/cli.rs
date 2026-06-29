use std::time::Duration;

use edcb_mcp::{
    PluginKind,
    cli::{CliAction, CliCommand, CliInvocation, OutputMode, format_services_plain},
    types::ServiceInfo,
};

fn empty_env() -> std::iter::Empty<(&'static str, &'static str)> {
    std::iter::empty()
}

#[test]
fn parses_global_flags_env_and_recorded_get_command() {
    let action = CliAction::from_args_and_env(
        [
            "edcb",
            "--host",
            "cli-host",
            "--port",
            "4511",
            "--timeout-seconds",
            "2",
            "--json",
            "recorded",
            "get",
            "42",
        ],
        [
            ("EDCB_HOST", "env-host"),
            ("EDCB_PORT", "4510"),
            ("EDCB_TIMEOUT_SECONDS", "9"),
        ],
    )
    .expect("valid CLI invocation should parse");

    assert_eq!(
        action,
        CliAction::Run(CliInvocation {
            host: "cli-host".to_string(),
            port: 4511,
            timeout: Duration::from_secs(2),
            output: OutputMode::Json,
            command: CliCommand::RecordedGet(42),
        })
    );
}

#[test]
fn parses_plain_plugin_command_without_cli_suffix() {
    let action =
        CliAction::from_args_and_env(["edcb", "plugins", "write", "--plain"], empty_env()).unwrap();

    assert_eq!(
        action,
        CliAction::Run(CliInvocation {
            host: "127.0.0.1".to_string(),
            port: 4510,
            timeout: Duration::from_secs(15),
            output: OutputMode::Plain,
            command: CliCommand::Plugins(PluginKind::Write),
        })
    );
}

#[test]
fn help_ignores_other_arguments() {
    let action =
        CliAction::from_args_and_env(["edcb", "--help", "--host", "bad", "services"], empty_env())
            .unwrap();

    assert_eq!(action, CliAction::Help);
}

#[test]
fn invalid_usage_uses_exit_code_2() {
    let error = CliAction::from_args_and_env(["edcb", "recorded", "get", "not-an-id"], empty_env())
        .expect_err("invalid info id should fail");

    assert_eq!(error.exit_code, 2);
    assert!(error.message.contains("info-id"));
}

#[test]
fn formats_services_as_stable_plain_lines() {
    let services = [ServiceInfo {
        onid: 32736,
        tsid: 32736,
        sid: 1024,
        service_type: 1,
        partial_reception_flag: 0,
        service_provider_name: String::new(),
        service_name: "NHK総合1・東京".to_string(),
        network_name: "関東広域0".to_string(),
        ts_name: "NHK総合・東京".to_string(),
        remote_control_key_id: 1,
    }];

    assert_eq!(
        format_services_plain(&services),
        "32736\t32736\t1024\t1\tNHK総合1・東京\n"
    );
}
