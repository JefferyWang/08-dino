mod bundle;

use anyhow::Result;

pub type ModulePath = String;
pub type ModuleSource = String;

/// Defines the interface of a module loader.
pub trait ModuleLoader {
    fn load(&self, specifier: &str) -> Result<ModuleSource>;
    fn resolve(&self, base: Option<&str>, specifier: &str) -> Result<ModulePath>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use bundle::run_bundle;

    #[test]
    fn bundle_ts_should_work() -> Result<()> {
        let ret = run_bundle("fixtures/main.ts", &Default::default())?;
        assert_eq!(ret, "(function(){async function execute(name){console.log(\"Executing lib.ts\");return`Hello ${name}!`;}async function main(){console.log(\"Executing main.ts\");console.log(await execute(\"world\"));}return{default:main};})();");
        Ok(())
    }
}
