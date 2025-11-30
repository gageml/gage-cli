use cursive::utils::markup::StyledString;

pub type Help = Vec<HelpSection>;

pub type HelpSection = (Option<&'static str>, Vec<KeyHelp>);

pub type KeyHelp = (&'static str, StyledString);
