pub struct LanguageInfo {
    pub display_name: &'static str,
    pub icon_slug: &'static str,
}

pub fn get_language_info(language_id: &str, file_name: &str) -> LanguageInfo {
    let ext = file_name
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase())
        .unwrap_or_default();

    let lang_lower = language_id.to_lowercase();

    // Match by known language_id or file extension
    match (lang_lower.as_str(), ext.as_str()) {
        ("rust", _) | (_, "rs") => LanguageInfo {
            display_name: "Rust",
            icon_slug: "rust",
        },
        ("python", _) | (_, "py") => LanguageInfo {
            display_name: "Python",
            icon_slug: "python",
        },
        ("javascript", _) | (_, "js") | (_, "mjs") | (_, "cjs") => LanguageInfo {
            display_name: "JavaScript",
            icon_slug: "javascript",
        },
        ("typescript", _) | (_, "ts") | (_, "mts") | (_, "cts") => LanguageInfo {
            display_name: "TypeScript",
            icon_slug: "typescript",
        },
        ("tsx", _) | (_, "tsx") => LanguageInfo {
            display_name: "TypeScript React",
            icon_slug: "react",
        },
        ("jsx", _) | (_, "jsx") => LanguageInfo {
            display_name: "JavaScript React",
            icon_slug: "react",
        },
        ("go", _) | (_, "go") => LanguageInfo {
            display_name: "Go",
            icon_slug: "go",
        },
        ("c", _) | (_, "c") | (_, "h") => LanguageInfo {
            display_name: "C",
            icon_slug: "c",
        },
        ("cpp", _) | ("c++", _) | (_, "cpp") | (_, "cc") | (_, "cxx") | (_, "hpp") => LanguageInfo {
            display_name: "C++",
            icon_slug: "cpp",
        },
        ("csharp", _) | ("c#", _) | (_, "cs") => LanguageInfo {
            display_name: "C#",
            icon_slug: "csharp",
        },
        ("java", _) | (_, "java") => LanguageInfo {
            display_name: "Java",
            icon_slug: "java",
        },
        ("kotlin", _) | (_, "kt") | (_, "kts") => LanguageInfo {
            display_name: "Kotlin",
            icon_slug: "kotlin",
        },
        ("swift", _) | (_, "swift") => LanguageInfo {
            display_name: "Swift",
            icon_slug: "swift",
        },
        ("ruby", _) | (_, "rb") => LanguageInfo {
            display_name: "Ruby",
            icon_slug: "ruby",
        },
        ("php", _) | (_, "php") => LanguageInfo {
            display_name: "PHP",
            icon_slug: "php",
        },
        ("html", _) | (_, "html") | (_, "htm") => LanguageInfo {
            display_name: "HTML",
            icon_slug: "html",
        },
        ("css", _) | (_, "css") => LanguageInfo {
            display_name: "CSS",
            icon_slug: "css",
        },
        ("scss", _) | (_, "scss") | (_, "sass") => LanguageInfo {
            display_name: "SCSS",
            icon_slug: "sass",
        },
        ("json", _) | ("jsonc", _) | (_, "json") | (_, "jsonc") => LanguageInfo {
            display_name: "JSON",
            icon_slug: "json",
        },
        ("toml", _) | (_, "toml") => LanguageInfo {
            display_name: "TOML",
            icon_slug: "toml",
        },
        ("yaml", _) | (_, "yaml") | (_, "yml") => LanguageInfo {
            display_name: "YAML",
            icon_slug: "yaml",
        },
        ("markdown", _) | ("markdown-inline", _) | (_, "md") | (_, "markdown") => LanguageInfo {
            display_name: "Markdown",
            icon_slug: "markdown",
        },
        ("shell script", _) | ("bash", _) | ("sh", _) | (_, "sh") | (_, "bash") | (_, "zsh") => LanguageInfo {
            display_name: "Shell",
            icon_slug: "bash",
        },
        ("dockerfile", _) | (_, "dockerfile") => LanguageInfo {
            display_name: "Dockerfile",
            icon_slug: "docker",
        },
        ("sql", _) | (_, "sql") => LanguageInfo {
            display_name: "SQL",
            icon_slug: "sql",
        },
        ("zig", _) | (_, "zig") => LanguageInfo {
            display_name: "Zig",
            icon_slug: "zig",
        },
        ("lua", _) | (_, "lua") => LanguageInfo {
            display_name: "Lua",
            icon_slug: "lua",
        },
        ("dart", _) | (_, "dart") => LanguageInfo {
            display_name: "Dart",
            icon_slug: "dart",
        },
        ("elixir", _) | (_, "ex") | (_, "exs") => LanguageInfo {
            display_name: "Elixir",
            icon_slug: "elixir",
        },
        ("erlang", _) | (_, "erl") => LanguageInfo {
            display_name: "Erlang",
            icon_slug: "erlang",
        },
        ("haskell", _) | (_, "hs") => LanguageInfo {
            display_name: "Haskell",
            icon_slug: "haskell",
        },
        ("nix", _) | (_, "nix") => LanguageInfo {
            display_name: "Nix",
            icon_slug: "nix",
        },
        ("scala", _) | (_, "scala") => LanguageInfo {
            display_name: "Scala",
            icon_slug: "scala",
        },
        ("vue", _) | (_, "vue") => LanguageInfo {
            display_name: "Vue",
            icon_slug: "vue",
        },
        ("svelte", _) | (_, "svelte") => LanguageInfo {
            display_name: "Svelte",
            icon_slug: "svelte",
        },
        ("graphql", _) | (_, "graphql") | (_, "gql") => LanguageInfo {
            display_name: "GraphQL",
            icon_slug: "graphql",
        },
        ("xml", _) | (_, "xml") => LanguageInfo {
            display_name: "XML",
            icon_slug: "xml",
        },
        _ => LanguageInfo {
            display_name: "Source Code",
            icon_slug: "zed",
        },
    }
}
