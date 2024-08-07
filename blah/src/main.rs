fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let security_baseline = baseline::load("/workspaces/azure-osconfig-v2/baseline/securitybaseline.yml")?;

    let status = security_baseline.check_status();
    log::info!("Status: {:?}", status);

    Ok(())
}
