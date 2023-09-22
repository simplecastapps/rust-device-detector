// use anyhow::Result;
// use serde_yaml::Value;
//
// use crate::utils;
//
// #[test]
// fn test_parser_shell_tvs() -> Result<()> {
//     let dd = &utils::DD;
//
//     let ua = "Leff Shell LC390TA2A";
//     let dd_res = dd.parse(ua, None)?;
//
//     assert!(!dd_res.is_bot(), "expected not bot");
//     assert!(dd_res.is_shell_tv());
//
//     let ua = "Leff Shell";
//     let dd_res = dd.parse(ua, None)?;
//
//     assert!(!dd_res.is_bot(), "expected not bot");
//     assert!(!dd_res.is_shell_tv());
//
//     Ok(())
// }
