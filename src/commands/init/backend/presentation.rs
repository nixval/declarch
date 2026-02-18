use super::BackendMeta;

pub(super) fn print_backend_meta(meta: &BackendMeta) {
    println!();
    if !meta.title.is_empty() && meta.title != "-" {
        println!("  Title:       {}", meta.title);
    }
    if !meta.description.is_empty() && meta.description != "-" {
        println!("  Description: {}", meta.description);
    }
    if !meta.maintainers.is_empty() {
        println!("  Maintainer:  {}", meta.maintainers.join(", "));
    } else if let Some(author) = &meta.author
        && author != "-"
    {
        println!("  Author:      {}", author);
    }
    if !meta.homepage.is_empty() && meta.homepage != "-" {
        println!("  Homepage:    {}", meta.homepage);
    }
    if let Some(guide) = &meta.installation_guide
        && guide != "-"
    {
        println!("  Install:     {}", guide);
    }
    if !meta.platforms.is_empty() {
        println!("  Platforms:   {}", meta.platforms.join(", "));
    }
    if !meta.requires.is_empty() {
        println!("  Requires:    {}", meta.requires.join(", "));
    }
    println!();
}
