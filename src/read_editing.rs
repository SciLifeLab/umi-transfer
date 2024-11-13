
#[derive(clap::ValueEnum, Clone,Debug)]
pub enum UMIDestination {
   Header,
   Inline,
}