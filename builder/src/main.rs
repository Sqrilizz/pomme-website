use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SITE_URL: &str = "https://pomme.rs";

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GithubReleaseAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GithubRelease {
    tag_name: String,
    published_at: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GithubRepo {
    stargazers_count: u64,
    forks_count: u64,
    open_issues_count: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Contributor {
    login: String,
    avatar_url: String,
    html_url: String,
    contributions: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CommitAuthorInfo {
    login: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CommitDetail {
    message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GithubCommit {
    sha: String,
    html_url: String,
    commit: CommitDetail,
    author: Option<CommitAuthorInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CachedData {
    repo: GithubRepo,
    release: GithubRelease,
    checksums: HashMap<String, String>,
    contributors: Vec<Contributor>,
    commits: Vec<GithubCommit>,
}

fn resolve_base_dir() -> PathBuf {
    if Path::new("builder").is_dir() {
        PathBuf::from(".")
    } else {
        PathBuf::from("..")
    }
}

fn cache_path() -> PathBuf {
    let base = resolve_base_dir();
    if base == Path::new(".") {
        PathBuf::from("builder/cached_data.json")
    } else {
        PathBuf::from("cached_data.json")
    }
}

fn fetch_real_data() -> (
    GithubRepo,
    GithubRelease,
    HashMap<String, String>,
    Vec<Contributor>,
    Vec<GithubCommit>,
) {
    let client = reqwest::blocking::Client::builder()
        .user_agent("pomme-site-builder")
        .build()
        .expect("failed to build http client");

    let repo_res = client
        .get("https://api.github.com/repos/PommeMC/Client")
        .send();
    let release_res = client
        .get("https://api.github.com/repos/PommeMC/Client/releases/latest")
        .send();

    if let (Ok(r_resp), Ok(rel_resp)) = (repo_res, release_res) {
        if r_resp.status().is_success() && rel_resp.status().is_success() {
            if let (Ok(repo), Ok(release)) = (
                r_resp.json::<GithubRepo>(),
                rel_resp.json::<GithubRelease>(),
            ) {
                let mut checksums = HashMap::new();
                if let Ok(sums_res) = client
                    .get(format!(
                        "https://github.com/PommeMC/Client/releases/download/{}/sha256sums.txt",
                        release.tag_name
                    ))
                    .send()
                {
                    if let Ok(text) = sums_res.text() {
                        for line in text.lines() {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let hash = parts[0];
                                let file = parts[1].trim_start_matches('*');
                                checksums.insert(file.to_string(), hash.to_string());
                            }
                        }
                    }
                }

                let contributors: Vec<Contributor> = client
                    .get("https://api.github.com/repos/PommeMC/Client/contributors?per_page=100")
                    .send()
                    .map(|r| r.json().unwrap_or_default())
                    .unwrap_or_default();

                let commits: Vec<GithubCommit> = client
                    .get("https://api.github.com/repos/PommeMC/Client/commits?per_page=7")
                    .send()
                    .map(|r| r.json().unwrap_or_default())
                    .unwrap_or_default();

                if !contributors.is_empty() && !commits.is_empty() {
                    let cache = CachedData {
                        repo: repo.clone(),
                        release: release.clone(),
                        checksums: checksums.clone(),
                        contributors: contributors.clone(),
                        commits: commits.clone(),
                    };
                    if let Ok(json_str) = serde_json::to_string_pretty(&cache) {
                        let _ = fs::write(cache_path(), json_str);
                    }
                    return (repo, release, checksums, contributors, commits);
                }
            }
        }
    }

    println!("GitHub API rate limit hit or offline, loading verified cached data...");
    let cached_str = fs::read_to_string(cache_path()).expect("failed to read cached_data.json");
    let cache: CachedData =
        serde_json::from_str(&cached_str).expect("failed to parse cached_data.json");
    (
        cache.repo,
        cache.release,
        cache.checksums,
        cache.contributors,
        cache.commits,
    )
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn render_layout(
    title: &str,
    desc: &str,
    current_page: &str,
    version: &str,
    content: &str,
) -> String {
    let nav_links = [
        ("/", "Overview"),
        ("/downloads", "Downloads"),
        ("/parity", "Vanilla Parity"),
        ("/architecture", "Architecture"),
        ("/community", "Contributors"),
    ];

    let mut links_html = String::new();
    for (href, label) in nav_links {
        let active =
            if current_page == href || (href == "/community" && current_page == "/contributors") {
                " active"
            } else {
                ""
            };
        let full_href = href;
        links_html.push_str(&format!(
            r#"<li><a href="{}" class="nav-link{}">{}</a></li>"#,
            full_href, active, label
        ));
    }

    let canonical_path = if current_page == "/" {
        String::new()
    } else {
        format!("{}/", current_page)
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{} | Pomme Minecraft Client (Rust &amp; Vulkan)</title>
  <meta name="description" content="{}">
  <meta name="theme-color" content="#08090d">
  <link rel="canonical" href="{}/{}">
  <link rel="icon" type="image/x-icon" href="/favicon.ico">
  <link rel="icon" type="image/png" href="/assets/brand/pomme.png">
  <link rel="apple-touch-icon" href="/assets/brand/pomme.png">
  <meta property="og:type" content="website">
  <meta property="og:site_name" content="Pomme">
  <meta property="og:title" content="{} | Pomme Minecraft Client">
  <meta property="og:description" content="{}">
  <meta property="og:url" content="{}/{}">
  <meta property="og:image" content="{}/assets/brand/pomme.png">
  <meta name="twitter:card" content="summary_large_image">
  <meta name="twitter:title" content="{} | Pomme Minecraft Client">
  <meta name="twitter:description" content="{}">
  <meta name="twitter:image" content="{}/assets/brand/pomme.png">
  <link rel="stylesheet" href="/assets/style.css">
</head>
<body>
  <header class="header">
    <div class="container nav">
      <a href="/" class="nav-brand">
        <img src="/assets/brand/pomme.png" alt="Pomme Logo" width="30" height="30">
        <span>Pomme</span>
        <span class="nav-brand-tag">v{}</span>
      </a>
      <ul class="nav-links">
        {}
      </ul>
      <div class="nav-actions">
        <a href="https://github.com/sponsors/Purdze" class="btn btn-secondary btn-sm mono nav-sponsor-btn" target="_blank" rel="noreferrer">
          Sponsor
        </a>
        <a href="https://github.com/PommeMC/Client" class="btn btn-secondary btn-sm mono" target="_blank" rel="noreferrer">
          GitHub
        </a>
        <a href="/downloads/" class="btn btn-primary btn-sm">
          Download
        </a>
      </div>
    </div>
  </header>

  <main>
    {}
  </main>

  <footer class="footer">
    <div class="container footer-inner">
      <div>
        <p class="mono" style="color: var(--text-muted); font-size: 0.8rem;">PommeMC &copy; 2026. Licensed under GPL-3.0.</p>
        <p style="font-size: 0.75rem; margin-top: 0.25rem;">Not affiliated with Mojang Studios or Microsoft. Minecraft is a trademark of Mojang Studios.</p>
        <p class="mono" style="font-size: 0.78rem; margin-top: 0.5rem; color: var(--text-dim);">Made with ❤️ by <a href="https://github.com/sqrilizz" target="_blank" rel="noreferrer" style="color: var(--text-muted); text-decoration: underline;">sqrilizz</a> and <a href="/community/" style="color: var(--text-muted); text-decoration: underline;">contributors</a></p>
      </div>
      <div class="footer-nav mono">
        <a href="/community/">Contributors</a>
        <a href="https://discord.gg/ucBA55bHPR" target="_blank" rel="noreferrer">Discord</a>
        <a href="https://github.com/PommeMC/Client" target="_blank" rel="noreferrer">GitHub</a>
        <a href="https://github.com/sponsors/Purdze" target="_blank" rel="noreferrer" style="color: var(--accent-green); font-weight: 700;">Sponsor Project &rarr;</a>
      </div>
    </div>
  </footer>
  <script src="/assets/app.js" defer></script>
</body>
</html>"##,
        title,
        desc,
        SITE_URL,
        canonical_path,
        title,
        desc,
        SITE_URL,
        canonical_path,
        SITE_URL,
        title,
        desc,
        SITE_URL,
        version,
        links_html,
        content
    )
}

fn build_index(repo: &GithubRepo, commits: &[GithubCommit], version: &str) -> String {
    let mut commits_html = String::new();
    for (idx, c) in commits.iter().enumerate() {
        let raw_title = c.commit.message.lines().next().unwrap_or("commit");
        let safe_title = escape_html(raw_title);

        let short_sha = if c.sha.len() >= 7 {
            &c.sha[..7]
        } else {
            &c.sha
        };
        let author_login = c
            .author
            .as_ref()
            .and_then(|a| a.login.clone())
            .unwrap_or_else(|| "contributor".to_string());
        let safe_author = escape_html(&author_login);

        let head_tag = if idx == 0 {
            r#"<span class="git-ref-head">HEAD -> master</span>"#
        } else {
            ""
        };

        commits_html.push_str(&format!(
            r#"<a href="{}" target="_blank" rel="noreferrer" class="git-log-row">
  <span class="git-graph-node">*</span>
  <span class="git-log-sha">{}</span>
  {}
  <span class="git-log-msg">{}</span>
  <span class="git-log-author">@{}</span>
</a>"#,
            c.html_url, short_sha, head_tag, safe_title, safe_author
        ));
    }

    format!(
        r#"<div class="hero">
  <div class="hero-watermark" aria-hidden="true"></div>
  <div class="container hero-grid">
    <div>
      <div class="hero-label">
        <span class="pulse-dot"></span>
        <span>STABLE RELEASE</span>
        <span>•</span>
        <span>v{}</span>
      </div>
      <h1 class="hero-title">Minecraft client,<br>written from scratch in <span class="badge-highlight">Rust</span>.</h1>
      <p class="hero-desc">
        Zero Mojang code. Zero Java runtime. One native binary to run. Desktop, Android and iOS running directly on Vulkan and Metal.
      </p>
      <div class="hero-ctas">
        <a href="/downloads/" class="btn btn-primary btn-lg">Get Release (v{})</a>
        <a href="https://github.com/PommeMC/Client" class="btn btn-secondary btn-lg mono" target="_blank" rel="noreferrer">
          git clone PommeMC/Client
        </a>
      </div>
      <div class="hero-stats">
        <div>
          <div class="hero-stat-value" data-counter="{}">{}</div>
          <div class="hero-stat-label">GitHub Stars</div>
        </div>
        <div>
          <div class="hero-stat-value">&lt; <span data-counter="300">300</span> MB</div>
          <div class="hero-stat-label">RAM Consumption</div>
        </div>
        <div>
          <div class="hero-stat-value">0 ms</div>
          <div class="hero-stat-label">JVM GC Pauses</div>
        </div>
      </div>
    </div>

    <div class="hero-col-right">
      <div class="terminal-card boot-race" aria-label="Cold start benchmark: Pomme launches in 1.4 seconds, Vanilla Java in 11.8 seconds, Lunar Client in 14.5 seconds">
        <div class="terminal-header">
          <div class="terminal-dots">
            <div class="terminal-dot"></div>
            <div class="terminal-dot"></div>
            <div class="terminal-dot"></div>
          </div>
          <div class="terminal-title">pomme-bench --cold-start</div>
          <button id="bootRaceReplay" class="term-copy-btn" type="button">Run again</button>
        </div>
        <div class="terminal-body">
          <div class="term-comment"># Launch until main menu is fully interactive</div>
          <div class="race-row" data-time="14.5">
            <span class="race-name term-dim">lunar-client</span>
            <span class="race-bar"><span class="race-fill race-fill-slow"></span></span>
            <span class="race-val mono term-dim">14.5s</span>
          </div>
          <div class="race-row" data-time="11.8">
            <span class="race-name term-dim">vanilla-java</span>
            <span class="race-bar"><span class="race-fill race-fill-slow"></span></span>
            <span class="race-val mono term-dim">11.8s</span>
          </div>
          <div class="race-row race-winner" data-time="1.4">
            <span class="race-name term-rust">pomme-rust</span>
            <span class="race-bar"><span class="race-fill race-fill-fast"></span></span>
            <span class="race-val mono term-highlight">1.4s</span>
          </div>
          <div class="race-result mono" id="raceResult" aria-live="polite">&nbsp;</div>
        </div>
      </div>
    </div>
  </div>
</div>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Engine capabilities</div>
      <h2 class="section-title">What Pomme is <span class="badge-highlight">today</span></h2>
      <p class="section-subtitle">Real architectural choices and verified runtime performance, not marketing promises.</p>
    </div>

    <div class="brutal-grid">
      <div class="brutal-card">
        <span class="brutal-metric">No JVM</span>
        <p>One native executable. No separate launcher, no JVM to install, no GC tuning flags. <span>Signs in with Microsoft, downloads assets, runs anywhere.</span></p>
      </div>

      <div class="brutal-card">
        <span class="brutal-metric">&lt; 300 MB</span>
        <p>Resident memory at 16 chunks render distance. <span>Vanilla Java sits near 2,410 MB before warmup even finishes.</span></p>
      </div>

      <div class="brutal-card">
        <span class="brutal-metric">0 ms GC</span>
        <p>Zero garbage collector micro-stutters during gameplay. <span>Deterministic memory arenas instead of random stop-the-world pauses.</span></p>
      </div>

      <div class="brutal-card">
        <span class="brutal-metric">1.20 – 26.3</span>
        <p>Multi-version server connection support. <span>Connects to any standard vanilla Java server with compiled dispatch tables.</span></p>
      </div>

      <div class="brutal-card">
        <span class="brutal-metric">5 Platforms</span>
        <p>Desktop and Mobile on one shared Rust codebase. <span>Windows, Linux, macOS, Android (ARM64 APK) and iOS (Metal via MoltenVK).</span></p>
      </div>

      <div class="brutal-card">
        <span class="brutal-metric">GPL-3.0</span>
        <p>100% Free and Open Source. <span>No telemetry, no paywalled cosmetic tiers. Read every line of code, fork it, audit it, contribute.</span></p>
      </div>
    </div>

    <div class="notice-box">
      <span class="notice-badge">Active development</span>
      <p>
        <strong>Pomme is under active development.</strong> While Vulkan rendering, singleplayer worlds (SteelMC), and standard multiplayer from 1.20 to 26.3 are fully playable, complex mechanics and shaderpacks are continuously shipping. Test it, report bugs on <a href="https://github.com/PommeMC/Client/issues" target="_blank" rel="noreferrer">GitHub</a>, and join <a href="https://discord.gg/ucBA55bHPR" target="_blank" rel="noreferrer">Discord</a> to follow daily commits. Full per-feature status lives in the <a href="/parity/">vanilla parity tracker</a>.
      </p>
    </div>
  </div>
</section>
<section class="section" style="padding-top: 1.5rem; padding-bottom: 2rem;">
  <div class="container">
    <div class="section-head text-center" style="max-width: 860px; margin-left: auto; margin-right: auto; text-align: center;">
      <div class="section-kicker">Visual Fidelity</div>
      <h2 class="section-title">Direct Vulkan <span class="badge-highlight">In-Game Render</span></h2>
      <p class="section-subtitle" style="margin-left: auto; margin-right: auto;">
        Real framebuffer captures from native Pomme builds. Hardware frustum culling, dynamic lightmaps, and zero stop-the-world GC pauses.
      </p>
    </div>

    <div class="gallery-viewport-card">
      <div class="gallery-topbar">
        <div class="terminal-dots">
          <div class="terminal-dot"></div>
          <div class="terminal-dot"></div>
          <div class="terminal-dot"></div>
        </div>
        <div class="gallery-window-title mono">pomme-client</div>
      </div>
      <div class="gallery-video-wrap">
        <video autoplay loop muted playsinline poster="/assets/images/pomme_gameplay_poster.webp" class="gallery-main-video">
          <source src="/assets/images/pomme_gameplay_60fps.mp4" type="video/mp4">
        </video>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Core Subsystems</div>
      <h2 class="section-title">Engineered in <span class="badge-highlight">Rust</span> from day one</h2>
      <p class="section-subtitle">
        Pomme decomposes the client into decoupled, audit-friendly crates. Select a subsystem to inspect its architecture.
      </p>
    </div>

    <div class="subsystem-container">
      <div class="subsystem-tabs" role="tablist" aria-label="Engine subsystems">
        <button class="subsystem-tab active" role="tab" id="tab-vulkan" aria-controls="sub-vulkan" aria-selected="true" data-target="sub-vulkan">Vulkan Renderer</button>
        <button class="subsystem-tab" role="tab" id="tab-proto" aria-controls="sub-proto" aria-selected="false" tabindex="-1" data-target="sub-proto">Multiplayer Protocol</button>
        <button class="subsystem-tab" role="tab" id="tab-steel" aria-controls="sub-steel" aria-selected="false" tabindex="-1" data-target="sub-steel">Singleplayer (SteelMC)</button>
        <button class="subsystem-tab" role="tab" id="tab-peel" aria-controls="sub-peel" aria-selected="false" tabindex="-1" data-target="sub-peel">Peel Inspector</button>
        <button class="subsystem-tab" role="tab" id="tab-mobile" aria-controls="sub-mobile" aria-selected="false" tabindex="-1" data-target="sub-mobile">Mobile Engine</button>
      </div>

      <div class="subsystem-panes">
        <div class="subsystem-pane active" id="sub-vulkan" role="tabpanel" aria-labelledby="tab-vulkan">
          <div class="subsystem-desc-card">
            <span class="subsystem-crate-tag">pomme-gpu &amp; pomme-gpu-allocator</span>
            <h3 class="subsystem-title">Direct Vulkan 1.3 Pipeline</h3>
            <p class="subsystem-text">
              Custom SPIR-V compute and vertex pipelines without legacy OpenGL state machine overhead. On macOS and iOS, calls translate directly to Apple Metal through MoltenVK with zero CPU penalty.
            </p>
            <ul class="subsystem-points">
              <li>Hardware occlusion culling and dynamic chunk meshing</li>
              <li>Native anisotropic filtering and mipmapped terrain textures</li>
              <li>Predictable frame times with zero driver-level stall bubbles</li>
            </ul>
          </div>
          <div class="terminal-card">
            <div class="terminal-header">
              <div class="terminal-dots"><div class="terminal-dot"></div><div class="terminal-dot"></div><div class="terminal-dot"></div></div>
              <div class="terminal-title">pomme-gpu ~ compile &amp; init</div>
              <button class="term-copy-btn" data-copy="cargo check -p pomme-gpu">Copy</button>
            </div>
            <div class="terminal-body">
              <div><span class="term-cmd">$</span> cargo check -p pomme-gpu</div>
              <div class="term-dim">   Compiling pomme-gpu-allocator v0.2.10</div>
              <div class="term-dim">   Compiling pomme-gpu v0.2.10</div>
              <div class="term-highlight">    Finished `dev` profile in 1.42s</div>
              <br>
              <div><span class="term-rust">[pomme-gpu]</span> <span class="term-dim">Physical device: Vulkan 1.3, 16 queue families</span></div>
              <div><span class="term-rust">[pomme-gpu]</span> <span class="term-dim">Memory budget: 8,192 MB allocated / 248 MB active</span></div>
              <div><span class="term-cmd">[pomme-gpu]</span> <span class="term-highlight">Present mode: VK_PRESENT_MODE_FIFO_KHR</span></div>
            </div>
          </div>
        </div>

        <div class="subsystem-pane" id="sub-proto" role="tabpanel" aria-labelledby="tab-proto" hidden>
          <div class="subsystem-desc-card">
            <span class="subsystem-crate-tag">pomme-protocol &amp; pomme-auth</span>
            <h3 class="subsystem-title">Dynamic Protocol Negotiation</h3>
            <p class="subsystem-text">
              Connect to any standard vanilla Java server from 1.20 to 26.3 snapshots. Zero-allocation packet decoders translate protocol variations on the wire with compiled dispatch tables.
            </p>
            <ul class="subsystem-points">
              <li>Automatic packet rewrite across 1.20, 1.20.4, 1.21 and 26.3</li>
              <li>Microsoft Xbox Live token verification using native OS Keyring</li>
              <li>Streaming chunk decompression on worker thread pools</li>
            </ul>
          </div>
          <div class="terminal-card">
            <div class="terminal-header">
              <div class="terminal-dots"><div class="terminal-dot"></div><div class="terminal-dot"></div><div class="terminal-dot"></div></div>
              <div class="terminal-title">pomme-protocol ~ server connect</div>
              <button class="term-copy-btn" data-copy="pomme-client --connect mc.example.net:25565">Copy</button>
            </div>
            <div class="terminal-body">
              <div><span class="term-cmd">$</span> pomme-client --connect mc.example.net:25565</div>
              <div class="term-dim">[pomme-protocol] Server protocol: 765 (Minecraft 1.20.4)</div>
              <div><span class="term-rust">[pomme-protocol]</span> <span class="term-dim">Handshake -> Login -> Play transition</span></div>
              <div><span class="term-rust">[pomme-auth]</span> <span class="term-highlight">Microsoft Xbox token validated</span></div>
              <div><span class="term-cmd">[pomme-protocol]</span> <span class="term-dim">World chunks streaming: 24 chunks/tick</span></div>
            </div>
          </div>
        </div>

        <div class="subsystem-pane" id="sub-steel" role="tabpanel" aria-labelledby="tab-steel" hidden>
          <div class="subsystem-desc-card">
            <span class="subsystem-crate-tag">pomme-singleplayer &amp; steelmc</span>
            <h3 class="subsystem-title">Embedded Singleplayer Engine</h3>
            <p class="subsystem-text">
              Singleplayer worlds run on an embedded multithreaded SteelMC server engine in the same process. Worlds start in under 500 ms with full Anvil chunk persistence and zero JVM overhead.
            </p>
            <ul class="subsystem-points">
              <li>Parallel chunk generation and entity ticking across all CPU cores</li>
              <li>No JVM startup or auxiliary local port forwarding</li>
              <li>Seamless compatibility with standard vanilla .mca region saves</li>
            </ul>
          </div>
          <div class="terminal-card">
            <div class="terminal-header">
              <div class="terminal-dots"><div class="terminal-dot"></div><div class="terminal-dot"></div><div class="terminal-dot"></div></div>
              <div class="terminal-title">pomme-singleplayer ~ world boot</div>
              <button class="term-copy-btn" data-copy="pomme-client --world Survival">Copy</button>
            </div>
            <div class="terminal-body">
              <div><span class="term-cmd">$</span> pomme-client --world Survival</div>
              <div class="term-dim">[pomme-singleplayer] Booting world "Survival"...</div>
              <div><span class="term-rust">[steelmc]</span> <span class="term-dim">Loading region r.0.0.mca: 1,024 chunks mapped</span></div>
              <div><span class="term-rust">[steelmc]</span> <span class="term-highlight">Spawn point calculated in 18 ms</span></div>
              <div><span class="term-cmd">[steelmc]</span> <span class="term-dim">Local player joined (zero-copy memory channel)</span></div>
            </div>
          </div>
        </div>

        <div class="subsystem-pane" id="sub-peel" role="tabpanel" aria-labelledby="tab-peel" hidden>
          <div class="subsystem-desc-card">
            <span class="subsystem-crate-tag">pomme-peel</span>
            <h3 class="subsystem-title">Real-Time Protocol Dissector</h3>
            <p class="subsystem-text">
              Built-in diagnostic inspection engine for dissecting network packets, capturing live sessions, and diffing behaviour against vanilla Minecraft for bug reproduction.
            </p>
            <ul class="subsystem-points">
              <li>Live packet capturing and payload inspection</li>
              <li>Tick-by-tick state diffing against vanilla protocol specs</li>
              <li>Direct export to diagnostic traces for GitHub issues</li>
            </ul>
          </div>
          <div class="terminal-card">
            <div class="terminal-header">
              <div class="terminal-dots"><div class="terminal-dot"></div><div class="terminal-dot"></div><div class="terminal-dot"></div></div>
              <div class="terminal-title">pomme-peel ~ packet inspect</div>
              <button class="term-copy-btn" data-copy="peel capture --port 25565">Copy</button>
            </div>
            <div class="terminal-body">
              <div><span class="term-cmd">$</span> peel capture --port 25565 --filter "clientbound/chunk_data"</div>
              <div class="term-dim">[peel] Capturing session #4192 (1.20.4)</div>
              <div><span class="term-rust">[peel]</span> <span class="term-dim">Packet 0x25: ChunkDataBiomes (x: 12, z: -4, size: 4,112 bytes)</span></div>
              <div><span class="term-cmd">[peel]</span> <span class="term-highlight">CRC verification: OK (0 errors detected)</span></div>
            </div>
          </div>
        </div>

        <div class="subsystem-pane" id="sub-mobile" role="tabpanel" aria-labelledby="tab-mobile" hidden>
          <div class="subsystem-desc-card">
            <span class="subsystem-crate-tag">pomme-android &amp; pomme-ios</span>
            <h3 class="subsystem-title">Cross-Platform Touch &amp; Metal</h3>
            <p class="subsystem-text">
              100% shared Rust codebase compiled for ARM64 Android (NativeActivity APK) and Apple iOS (Metal via MoltenVK) with full on-screen touch virtualization.
            </p>
            <ul class="subsystem-points">
              <li>On-screen virtual thumbstick, camera look, hotbar touch gestures</li>
              <li>Native Android Keyring and Apple iOS Keychain authentication</li>
              <li>Battery-aware Vulkan present queues and thermal scaling</li>
            </ul>
          </div>
          <div class="terminal-card">
            <div class="terminal-header">
              <div class="terminal-dots"><div class="terminal-dot"></div><div class="terminal-dot"></div><div class="terminal-dot"></div></div>
              <div class="terminal-title">pomme-mobile ~ build target</div>
              <button class="term-copy-btn" data-copy="just android-run">Copy</button>
            </div>
            <div class="terminal-body">
              <div><span class="term-cmd">$</span> just android-run</div>
              <div class="term-dim">[pomme-android] NativeActivity entrypoint initialized</div>
              <div><span class="term-rust">[pomme-android]</span> <span class="term-dim">Vulkan surface attached to ANativeWindow</span></div>
              <div><span class="term-rust">[pomme-auth]</span> <span class="term-highlight">OAuth device-code flow: https://microsoft.com/link</span></div>
              <div><span class="term-cmd">[pomme-input]</span> <span class="term-dim">Multi-touch subsystem ready</span></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head" style="display: flex; justify-content: space-between; align-items: flex-end; flex-wrap: wrap; gap: 1rem;">
      <div>
        <div class="section-kicker">Live development</div>
        <h2 class="section-title">Recent Git Commits</h2>
        <p class="section-subtitle">Real-time stream directly from the PommeMC/Client master branch.</p>
      </div>
      <a href="https://github.com/PommeMC/Client/commits/master" class="btn btn-secondary btn-sm mono" target="_blank" rel="noreferrer">
        GitHub Tree &rarr;
      </a>
    </div>

    <div class="git-log-terminal">
      <div class="terminal-header">
        <div class="terminal-dots">
          <div class="terminal-dot"></div>
          <div class="terminal-dot"></div>
          <div class="terminal-dot"></div>
        </div>
        <div class="terminal-title">git log --graph --oneline -n 7 master</div>
        <div style="display: flex; gap: 0.5rem; align-items: center;">
          <span class="git-branch-badge">origin/master</span>
          <button class="term-copy-btn" data-copy="git log --oneline -n 7">Copy cmd</button>
        </div>
      </div>
      <div class="git-log-body mono">
        {}
      </div>
      <div class="git-log-footer mono">
        <span><span class="pulse-dot" style="display: inline-block; width: 6px; height: 6px; margin-right: 6px;"></span>PommeMC/Client: synchronized with GitHub API</span>
        <a href="https://github.com/PommeMC/Client/commits/master" target="_blank" rel="noreferrer" class="git-view-all">Full commit history on GitHub &rarr;</a>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Multiplatform ecosystems</div>
      <h2 class="section-title">Desktop &amp; Mobile.</h2>
      <p class="section-subtitle">Pomme is engineered in portable Rust to run with full native performance across desktop and mobile devices.</p>
    </div>

    <div class="benchmarks-grid">
      <div class="bench-card">
        <h3 style="font-size: 1.15rem; margin-bottom: 0.5rem;">Desktop Native</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6; margin-bottom: 1rem;">
          Windows, Linux and macOS. Single standalone binary that signs in with Microsoft OAuth, downloads game assets and handles automatic updates directly.
        </p>
        <span class="badge-tag badge-green">Production Ready</span>
      </div>

      <div class="bench-card">
        <h3 style="font-size: 1.15rem; margin-bottom: 0.5rem;">Android (APK)</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6; margin-bottom: 1rem;">
          Native aarch64 ARM64 port via NativeActivity. Full touch controls (on-screen virtual joystick, touch camera, hotbar tapping), asset streaming and Microsoft sign-in.
        </p>
        <span class="badge-tag badge-rust">In Active Dev (Builds from source)</span>
      </div>

      <div class="bench-card">
        <h3 style="font-size: 1.15rem; margin-bottom: 0.5rem;">iOS (iPhone & iPad)</h3>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6; margin-bottom: 1rem;">
          100% Rust binary running via MoltenVK & OpenAL Soft. Touch controls, iOS Keychain credential storage, and TestFlight export support.
        </p>
        <span class="badge-tag badge-rust">In Active Dev (Builds from source)</span>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Integrity &amp; Licensing</div>
      <h2 class="section-title">Same inputs, verified <span class="badge-highlight">outputs</span></h2>
      <p class="section-subtitle">
        Pomme holds every subsystem to the exact inputs of Minecraft: Java Edition until runtime results match vanilla.
      </p>
    </div>

    <div class="guarantee-grid">
      <div class="guarantee-card">
        <h4>Your Own Legitimate Copy</h4>
        <p>
          Pomme requires an official Microsoft Minecraft Java Edition license. Microsoft OAuth tokens are negotiated directly with Xbox Live and stored in your native OS Keyring or Keychain.
        </p>
      </div>

      <div class="guarantee-card">
        <h4>Clean-Room Rust Implementation</h4>
        <p>
          Pomme is 100% free open-source code under GPL-3.0. It contains zero decompiled Mojang bytecode, zero leaked assets, and no proprietary binaries. Game textures and sounds stream from Mojang official CDN.
        </p>
      </div>

      <div class="guarantee-card">
        <h4>Differential Test Verification</h4>
        <p>
          Network packets, world palettes, and lighting calculations are verified with differential test harnesses and packet inspection tools (Peel) to guarantee seamless compatibility with vanilla servers.
        </p>
      </div>
    </div>
  </div>
</section>

<section class="section" style="padding-top: 2rem; border-bottom: none;">
  <div class="container" style="max-width: 920px;">
    <div class="try-box">
      <h2 style="font-size: clamp(1.8rem, 4vw, 2.5rem); font-weight: 800; color: var(--accent-green); margin-bottom: 0.75rem;">
        Try it next to your current client.
      </h2>
      <p style="color: var(--text-muted); font-size: 1.1rem; line-height: 1.6; margin-bottom: 2rem; max-width: 620px; margin-left: auto; margin-right: auto;">
        It is one standalone binary. Drop it in a folder, run it, and see what native Rust and Vulkan feel like on your hardware before you commit.
      </p>
      <div style="display: flex; gap: 1rem; justify-content: center; flex-wrap: wrap;">
        <a href="/downloads/" class="btn btn-primary btn-lg">Download Pomme</a>
        <a href="https://github.com/PommeMC/Client" class="btn btn-secondary btn-lg mono" target="_blank" rel="noreferrer">Star on GitHub</a>
        <a href="/contributors/" class="btn btn-secondary btn-lg">Meet the contributors</a>
      </div>
    </div>
  </div>
</section>"#,
        version, version, repo.stargazers_count, repo.stargazers_count, commits_html
    )
}

fn build_downloads(release: &GithubRelease, checksums: &HashMap<String, String>) -> String {
    let mut assets_map = HashMap::new();
    for a in &release.assets {
        assets_map.insert(a.name.as_str(), a);
    }

    let win_setup = assets_map.get("pomme-client-windows-x64-setup.exe");
    let win_zip = assets_map.get("pomme-client-windows-x64.zip");
    let mac_dmg = assets_map.get("pomme-client-macos-arm64.dmg");
    let mac_zip = assets_map.get("pomme-client-macos-arm64.zip");
    let linux_app = assets_map.get("pomme-client-linux-x64-gnu.AppImage");
    let linux_zip = assets_map.get("pomme-client-linux-x64-gnu.zip");

    let get_hash = |name: &str| -> String {
        checksums
            .get(name)
            .cloned()
            .unwrap_or_else(|| "Verification hash pending".to_string())
    };

    let win_hash = get_hash("pomme-client-windows-x64-setup.exe");
    let linux_hash = get_hash("pomme-client-linux-x64-gnu.AppImage");
    let mac_hash = get_hash("pomme-client-macos-arm64.dmg");

    let win_setup_url = win_setup
        .map(|a| a.browser_download_url.as_str())
        .unwrap_or("#");
    let win_setup_size = win_setup.map(|a| format_bytes(a.size)).unwrap_or_default();
    let win_zip_url = win_zip
        .map(|a| a.browser_download_url.as_str())
        .unwrap_or("#");
    let win_zip_size = win_zip.map(|a| format_bytes(a.size)).unwrap_or_default();

    let linux_app_url = linux_app
        .map(|a| a.browser_download_url.as_str())
        .unwrap_or("#");
    let linux_app_size = linux_app.map(|a| format_bytes(a.size)).unwrap_or_default();
    let linux_zip_url = linux_zip
        .map(|a| a.browser_download_url.as_str())
        .unwrap_or("#");
    let linux_zip_size = linux_zip.map(|a| format_bytes(a.size)).unwrap_or_default();

    let mac_dmg_url = mac_dmg
        .map(|a| a.browser_download_url.as_str())
        .unwrap_or("#");
    let mac_dmg_size = mac_dmg.map(|a| format_bytes(a.size)).unwrap_or_default();
    let mac_zip_url = mac_zip
        .map(|a| a.browser_download_url.as_str())
        .unwrap_or("#");
    let mac_zip_size = mac_zip.map(|a| format_bytes(a.size)).unwrap_or_default();

    format!(
        r##"<section class="section" style="padding-top: 4.5rem; padding-bottom: 2rem;">
  <div class="container" style="max-width: 820px; text-align: center; margin-left: auto; margin-right: auto;">
    <div class="section-kicker" style="margin-bottom: 0.75rem;">Official Releases</div>
    <h1 class="section-title" style="font-size: clamp(2.4rem, 5vw, 3.2rem); margin-bottom: 1rem;">
      Download <span class="badge-highlight">Pomme</span>
    </h1>
    <p class="section-subtitle" style="margin-left: auto; margin-right: auto; font-size: 1.15rem; color: var(--text-muted); line-height: 1.6; margin-bottom: 1.25rem;">
      Single native binary. No installer. No JVM. One click to run.
    </p>
    <div style="display: inline-flex; align-items: center; gap: 0.75rem; background: var(--bg-panel); border: 1px solid var(--border-subtle); padding: 0.4rem 1.1rem; border-radius: 999px;">
      <span class="pulse-dot"></span>
      <span class="mono" style="color: var(--accent-green); font-weight: 700; font-size: 0.85rem;">{}</span>
      <span style="color: var(--text-dim); font-size: 0.85rem;">&bull;</span>
      <span style="color: var(--text-dim); font-size: 0.825rem;">Released on GitHub</span>
    </div>
  </div>
</section>

<section class="section" style="padding-top: 1rem; padding-bottom: 4.5rem;">
  <div class="container">
    <div class="downloads-platform-grid">
      <div class="download-card" data-platform="windows">
        <div class="download-card-header">
          <div class="download-icon" style="margin-bottom: 0;">WIN64</div>
          <span class="badge-tag badge-green detected-badge" hidden>Recommended</span>
        </div>
        <div class="download-os">Windows</div>
        <div class="download-spec">Windows 10 / 11 (x86_64) &bull; Vulkan 1.2+</div>
        <div class="download-actions">
          <a href="{}" class="btn btn-primary" style="width: 100%;">Download Installer ({})</a>
          <a href="{}" class="btn btn-secondary btn-sm mono" style="width: 100%;">Portable .zip ({})</a>
        </div>
        <details class="verify-details" style="width: 100%; margin-top: 1.25rem;">
          <summary class="mono" style="font-size: 0.75rem; color: var(--text-dim); cursor: pointer;">Verify &bull; SHA256</summary>
          <div class="download-hash" style="margin-top: 0.5rem;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.35rem;">
              <span class="term-dim mono" style="font-size: 0.72rem;">SHA256</span>
              <button class="term-copy-btn" data-copy="{}">Copy</button>
            </div>
            <div class="mono" style="font-size: 0.7rem; color: var(--text-dim); word-break: break-all;">{}</div>
          </div>
        </details>
      </div>

      <div class="download-card recommended" data-platform="linux">
        <div class="download-card-header">
          <div class="download-icon" style="margin-bottom: 0;">LINUX</div>
          <span class="badge-tag badge-green detected-badge">Recommended</span>
        </div>
        <div class="download-os">Linux</div>
        <div class="download-spec">glibc 2.31+ &bull; x86_64 &bull; AppImage / Tarball</div>
        <div class="download-actions">
          <a href="{}" class="btn btn-primary" style="width: 100%;">Download AppImage ({})</a>
          <a href="{}" class="btn btn-secondary btn-sm mono" style="width: 100%;">Standalone .zip ({})</a>
        </div>
        <details class="verify-details" style="width: 100%; margin-top: 1.25rem;">
          <summary class="mono" style="font-size: 0.75rem; color: var(--text-dim); cursor: pointer;">Verify &bull; SHA256</summary>
          <div class="download-hash" style="margin-top: 0.5rem;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.35rem;">
              <span class="term-dim mono" style="font-size: 0.72rem;">SHA256</span>
              <button class="term-copy-btn" data-copy="{}">Copy</button>
            </div>
            <div class="mono" style="font-size: 0.7rem; color: var(--text-dim); word-break: break-all;">{}</div>
          </div>
        </details>
      </div>

      <div class="download-card" data-platform="macos">
        <div class="download-card-header">
          <div class="download-icon" style="margin-bottom: 0;">MACOS</div>
          <span class="badge-tag badge-green detected-badge" hidden>Recommended</span>
        </div>
        <div class="download-os">macOS</div>
        <div class="download-spec">macOS 12+ &bull; Apple Silicon (M1/M2/M3/M4)</div>
        <div class="download-actions">
          <a href="{}" class="btn btn-primary" style="width: 100%;">Download .dmg ({})</a>
          <a href="{}" class="btn btn-secondary btn-sm mono" style="width: 100%;">Portable .zip ({})</a>
        </div>
        <details class="verify-details" style="width: 100%; margin-top: 1.25rem;">
          <summary class="mono" style="font-size: 0.75rem; color: var(--text-dim); cursor: pointer;">Verify &bull; SHA256</summary>
          <div class="download-hash" style="margin-top: 0.5rem;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.35rem;">
              <span class="term-dim mono" style="font-size: 0.72rem;">SHA256</span>
              <button class="term-copy-btn" data-copy="{}">Copy</button>
            </div>
            <div class="mono" style="font-size: 0.7rem; color: var(--text-dim); word-break: break-all;">{}</div>
          </div>
        </details>
      </div>
    </div>

    <div style="max-width: 860px; margin: 3rem auto 0 auto;">
      <div class="terminal-card">
        <div class="terminal-header">
          <div class="terminal-dots">
            <div class="terminal-dot"></div>
            <div class="terminal-dot"></div>
            <div class="terminal-dot"></div>
          </div>
          <div class="terminal-title">terminal &mdash; quick launch &amp; source build</div>
          <button class="term-copy-btn" data-copy="curl -sL https://github.com/PommeMC/Client/releases/latest/download/pomme-client-linux-x64-gnu.AppImage -o pomme && chmod +x pomme && ./pomme">Copy</button>
        </div>
        <div class="terminal-body">
          <div class="term-dim">Quick launch standalone AppImage:</div>
          <div><span class="term-cmd">$</span> curl -sL https://github.com/PommeMC/Client/releases/latest/download/pomme-client-linux-x64-gnu.AppImage -o pomme &amp;&amp; chmod +x pomme &amp;&amp; ./pomme</div>
          <br>
          <div class="term-dim">Compile from source with Cargo:</div>
          <div><span class="term-cmd">$</span> git clone https://github.com/PommeMC/Client.git &amp;&amp; cd Client &amp;&amp; cargo build --release</div>
        </div>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="requirements-grid">
      <div>
        <div class="section-kicker">Hardware</div>
        <h2 class="section-title">System <span class="badge-highlight">Requirements</span></h2>
        <p class="section-subtitle" style="margin-top: 1rem; line-height: 1.7;">
          Pomme is lightweight by design. A Vulkan-capable GPU is the only real requirement &mdash; everything else is minimal. Zero JVM classloading, zero garbage collector stutter.
        </p>
      </div>

      <div>
        <div style="display: flex; flex-direction: column; gap: 0.85rem;">
          <div style="background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 1.1rem 1.4rem; display: flex; justify-content: space-between; align-items: center;">
            <span class="mono" style="font-weight: 700; color: var(--accent-rust); font-size: 0.85rem;">GPU</span>
            <span style="color: var(--text-main); font-size: 0.9rem;">Vulkan 1.2+ compatible (or Metal via MoltenVK)</span>
          </div>
          <div style="background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 1.1rem 1.4rem; display: flex; justify-content: space-between; align-items: center;">
            <span class="mono" style="font-weight: 700; color: var(--accent-rust); font-size: 0.85rem;">CPU</span>
            <span style="color: var(--text-main); font-size: 0.9rem;">x86_64 or ARM64 (64-bit)</span>
          </div>
          <div style="background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 1.1rem 1.4rem; display: flex; justify-content: space-between; align-items: center;">
            <span class="mono" style="font-weight: 700; color: var(--accent-rust); font-size: 0.85rem;">RAM</span>
            <span style="color: var(--text-main); font-size: 0.9rem;">256 MB minimum (&lt; 300 MB at 16 chunks)</span>
          </div>
          <div style="background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 1.1rem 1.4rem; display: flex; justify-content: space-between; align-items: center;">
            <span class="mono" style="font-weight: 700; color: var(--accent-rust); font-size: 0.85rem;">DISK</span>
            <span style="color: var(--text-main); font-size: 0.9rem;">~50 MB executable space</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Mobile Ports</div>
      <h2 class="section-title">Android &amp; iOS <span class="badge-highlight">Builds</span></h2>
      <p class="section-subtitle">
        Experimental mobile targets with touch controls and Microsoft account sign-in.
      </p>
    </div>

    <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 1.5rem;">
      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title">Android (arm64-v8a)</div>
          <span class="badge-tag badge-rust">Build from source</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6; margin-bottom: 1.25rem;">
          Builds as a standalone APK. Full on-screen touch controls (virtual joystick, touch camera, hotbar tapping), asset streaming and Microsoft sign-in via device link code.
        </p>
        <div class="terminal-body" style="background: var(--bg-inset); border-radius: 6px; padding: 1rem; margin-bottom: 1.25rem; position: relative;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
            <div class="term-dim">Build and install APK on device:</div>
            <button class="term-copy-btn" data-copy="just android-install && just android-run">Copy</button>
          </div>
          <div><span class="term-cmd">$</span> just android-install</div>
          <div><span class="term-cmd">$</span> just android-run</div>
        </div>
        <a href="https://github.com/PommeMC/Client/blob/master/pomme-android/README.md" class="btn btn-secondary btn-sm mono" target="_blank" rel="noreferrer">
          Android Build Guide &rarr;
        </a>
      </div>

      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title">iOS &amp; iPadOS (iOS 16+)</div>
          <span class="badge-tag badge-rust">Build from source</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6; margin-bottom: 1.25rem;">
          Runs via MoltenVK (Vulkan over Apple Metal). Native iOS Keychain authentication, same touch controls, and TestFlight archive support.
        </p>
        <div class="terminal-body" style="background: var(--bg-inset); border-radius: 6px; padding: 1rem; margin-bottom: 1.25rem; position: relative;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
            <div class="term-dim">Build Xcode project and run on device:</div>
            <button class="term-copy-btn" data-copy="just ios-device <apple-team-id>">Copy</button>
          </div>
          <div><span class="term-cmd">$</span> just ios-device &lt;apple-team-id&gt;</div>
          <div><span class="term-cmd">$</span> just ios-archive &lt;apple-team-id&gt;</div>
        </div>
        <a href="https://github.com/PommeMC/Client/blob/master/pomme-ios/README.md" class="btn btn-secondary btn-sm mono" target="_blank" rel="noreferrer">
          iOS Build Guide &rarr;
        </a>
      </div>
    </div>

    <div class="notice-box" style="margin-top: 3.5rem;">
      <span class="notice-badge">First Launch</span>
      <div style="font-size: 0.9rem; color: var(--text-muted); line-height: 1.7;">
        Because Pomme builds are community-compiled without commercial EV signing keys, <strong>Windows SmartScreen</strong> may ask for approval (<code class="mono" style="color: var(--accent-rust);">More info &rarr; Run anyway</code>) and macOS requires <code class="mono" style="color: var(--accent-rust);">Right-click &rarr; Open</code>. On Linux, run <code class="mono" style="color: var(--accent-rust);">chmod +x</code> on the AppImage. Sign-in uses official Microsoft OAuth with tokens saved in your OS Keyring.
      </div>
    </div>
  </div>
</section>"##,
        release.tag_name,
        win_setup_url,
        win_setup_size,
        win_zip_url,
        win_zip_size,
        win_hash,
        win_hash,
        linux_app_url,
        linux_app_size,
        linux_zip_url,
        linux_zip_size,
        linux_hash,
        linux_hash,
        mac_dmg_url,
        mac_dmg_size,
        mac_zip_url,
        mac_zip_size,
        mac_hash,
        mac_hash
    )
}

#[derive(Debug, serde::Deserialize)]
struct ParityDataset {
    meta: ParityMeta,
    items: Vec<ParityEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct ParityMeta {
    commit: String,
    commit_title: String,
    commit_date: String,
    tag: String,
    updated_at: String,
    test_count: usize,
}

#[derive(Debug, serde::Deserialize)]
struct ParityEntry {
    name: String,
    category: String,
    status: String,
    tag: String,
    description: String,
}

fn build_parity() -> String {
    let raw_json = include_str!("../../data/parity.json");
    let dataset: ParityDataset = serde_json::from_str(raw_json).expect("valid parity.json");
    let meta = dataset.meta;
    let features = dataset.items;

    let group_defs = [
        (
            "entities",
            "01",
            "Entities & Mobs",
            "Every passive animal, hostile monster, boss & vehicle in Minecraft",
        ),
        (
            "blocks",
            "02",
            "Blocks & Redstone",
            "Pistons, Redstone logic, observers, hoppers, crafter & fluids",
        ),
        (
            "workstations",
            "03",
            "Workstations & Storage",
            "Crafting table, furnaces, brewing, anvil, enchanting & shulker boxes",
        ),
        (
            "items",
            "04",
            "Items & Equipment",
            "Data components, bundles, tools, armor, shields & elytra",
        ),
        (
            "player",
            "05",
            "Player & Combat",
            "20 Hz physics, swimming, attack cooldown, sweeping edge & saturation",
        ),
        (
            "world",
            "06",
            "World & Dimensions",
            "Overworld, Nether, End, portals, biomes, smooth AO & SteelMC",
        ),
        (
            "ui",
            "07",
            "UI & Overlays",
            "HUD, boss bars, F3 debug screen, tablist, chat & spectator mode",
        ),
        (
            "engine",
            "08",
            "Engine, Shaders & Net",
            "Vulkan 1.3, Metal, Iris GLSL shaders, 1.20-26.3 protocol & audio",
        ),
    ];

    let status_chip = |status: &str| match status {
        "Ready" => ("badge-green", "Ready"),
        "In Progress" => ("badge-rust", "In&nbsp;Progress"),
        _ => ("badge-gray", "Planned"),
    };

    let total = features.len() as u64;
    let count_status = |s: &str| features.iter().filter(|f| f.status == s).count() as u64;
    let count_cat = |c: &str| features.iter().filter(|f| f.category == c).count() as u64;

    let ready = count_status("Ready");
    let in_progress = count_status("In Progress");
    let planned = count_status("Planned");

    let pct = |n: u64| ((n * 1000 + total / 2) / total) as f64 / 10.0;
    let ready_pct = pct(ready);
    let progress_pct = pct(in_progress);
    let planned_pct = 100.0 - ready_pct - progress_pct;

    let mut domain_cards_html = String::new();
    for (key, _, title, subtitle) in group_defs {
        let total_in_cat = features.iter().filter(|f| f.category == key).count();
        let ready_in_cat = features
            .iter()
            .filter(|f| f.category == key && f.status == "Ready")
            .count();
        let cat_pct = (ready_in_cat * 100).checked_div(total_in_cat).unwrap_or(0);
        domain_cards_html.push_str(&format!(
            r#"<button type="button" class="domain-card" data-domain="{}">
  <div class="domain-card-head">
    <span class="domain-card-title">{}</span>
    <span class="domain-card-ratio mono">{}/{}</span>
  </div>
  <div class="domain-card-bar">
    <div class="domain-card-bar-fill" style="width: {}%;"></div>
  </div>
  <div class="domain-card-desc">{}</div>
</button>"#,
            key, title, ready_in_cat, total_in_cat, cat_pct, subtitle
        ));
    }

    let mut groups_html = String::new();
    for (key, num, title, _) in group_defs {
        let mut group_items = String::new();
        for item in &features {
            if item.category != key {
                continue;
            }
            let (badge_class, status_label) = status_chip(&item.status);
            group_items.push_str(&format!(
                r#"<div class="tracker-item" data-category="{}" data-status="{}">
  <div class="tracker-info">
    <div class="tracker-item-head">
      <h4>{}</h4>
      <span class="tracker-crate mono">{}</span>
    </div>
    <p>{}</p>
  </div>
  <span class="badge-tag {} status-chip">{}</span>
</div>"#,
                item.category,
                item.status,
                item.name,
                item.tag,
                item.description,
                badge_class,
                status_label
            ));
        }

        let group_count = features.iter().filter(|f| f.category == key).count();

        groups_html.push_str(&format!(
            r#"<div class="tracker-group" data-group="{}">
  <h3 class="tracker-group-title mono"><span class="tracker-group-num">{}</span> {} <span class="tracker-group-count">({})</span></h3>
  <div class="tracker-grid">
    {}
  </div>
</div>"#,
            key, num, title, group_count, group_items
        ));
    }

    let cat_pills = [
        ("all", "All Categories"),
        ("entities", "Entities & Mobs"),
        ("blocks", "Blocks & Redstone"),
        ("workstations", "Workstations & Storage"),
        ("items", "Items & Gear"),
        ("player", "Player & Combat"),
        ("world", "World & Gen"),
        ("ui", "UI & Screens"),
        ("engine", "Engine & Shaders"),
    ];

    let mut cat_pills_html = String::new();
    for (key, label) in cat_pills {
        let count = if key == "all" { total } else { count_cat(key) };
        let active = if key == "all" { " active" } else { "" };
        cat_pills_html.push_str(&format!(
            r#"<button class="tracker-pill cat-pill{}" data-filter="{}">{} ({})</button>"#,
            active, key, label, count
        ));
    }

    let status_pills: [(&str, &str, u64); 4] = [
        ("all", "All statuses", total),
        ("Ready", "Ready", ready),
        ("In Progress", "In progress", in_progress),
        ("Planned", "Planned", planned),
    ];

    let mut status_pills_html = String::new();
    for (key, label, count) in status_pills {
        let active = if key == "all" { " active" } else { "" };
        status_pills_html.push_str(&format!(
            r#"<button class="tracker-pill status-pill{}" data-filter="{}">{} ({})</button>"#,
            active, key, label, count
        ));
    }

    let harnesses = [
        ("PASS", "blocks/ (13 Test Suites)", "Pistons, moving piston simulation (moving_piston), voxel collision shapes, signs with text screen, multipart conditions, brewing & crafter, and container screens tested against vanilla 26.2 server."),
        ("PASS", "entities/ (24 Test Suites)", "Passive breeding, specialized behaviors (bees, foxes, pandas, axolotls), boat models with passenger seats, 3D nameplates, villager schedules, and glowing outlines."),
        ("PASS", "items/ (11 Test Suites)", "1.20.5+ item data component deserialization, 3D player heads in menus, container item tooltips, bundle nesting inside shulker boxes, and equipment durability."),
        ("PASS", "player/ (14 Test Suites)", "Tick-exact 20 Hz movement, third-person camera culling, offhand shield deflections, footsteps per material, fall damage formulas, and spectator effect shaders."),
        ("PASS", "ui/ (10 Test Suites)", "Vanilla HUD heart/hunger/armor overlays, FPS counter & debug options, auto-closing chat on container open, advancement toast popups, and sidebar scoreboards."),
        ("PASS", "world/ (15 Test Suites)", "Deterministic chunk arena memory allocation, 4-vertex ambient occlusion lightmaps, dynamic weather precipitation, and Iris/OptiFine shaderpack pipeline."),
        ("PASS", "protocol/ (Multi-Version)", "Compiled version dispatch tables and zero-allocation packet rewrite connecting to servers from 1.20 to 26.3 with Microsoft Keyring authentication."),
        ("PASS", "singleplayer/ (SteelMC)", "In-process multithreaded Rust server, Anvil (.mca) region file serialization, round-trip compatible with vanilla Java saves without JVM or local TCP sockets."),
        ("RUN", "shaderpack/ (Iris GLSL Pipeline)", "GLSL directive parser, stage pipelines (shadow, gbuffers, composite, deferred), and uniform injection in pomme-client::shaderpack."),
        ("RUN", "protocol/chunk (Pre-1.21.5 Translation)", "Dynamic backward-compatibility packet decoders converting pre-1.21.5 paletted chunk formats into internal chunk representation."),
    ];
    let mut harness_rows = String::new();
    for (idx, (verdict, title, desc)) in harnesses.iter().enumerate() {
        let num = format!("{:02}", idx + 1);
        let verdict_class = if *verdict == "PASS" {
            "verdict-pass"
        } else {
            "verdict-run"
        };
        harness_rows.push_str(&format!(
            r#"<div class="verification-row">
  <span class="verdict mono {}">{}</span>
  <div class="verification-text">
    <h4>{} <span class="verification-num mono">#[{}]</span></h4>
    <p>{}</p>
  </div>
</div>"#,
            verdict_class, verdict, title, num, desc
        ));
    }

    format!(
        r#"<section class="section" style="padding-top: 3.5rem;">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Implementation Tracker</div>
      <h1 class="section-title">Vanilla Parity & Feature Status</h1>
      <p class="section-subtitle">
        Pomme tracks every Minecraft mechanic, mob, and block. Exactly {} entries cataloged across 8 domains with direct verification links. The full dataset lives in <code class="mono" style="color: var(--accent-rust);">parity.json</code>.
      </p>
    </div>

    <div class="domain-matrix">
      {}
    </div>

    <div class="parity-meta-card">
      <div class="parity-meta-row">
        <div class="parity-meta-item">
          <span class="parity-meta-label">Upstream Commit</span>
          <a href="https://github.com/PommeMC/Client/commit/{}" class="parity-meta-val mono" target="_blank" rel="noreferrer">{} &nearr;</a>
        </div>
        <div class="parity-meta-item">
          <span class="parity-meta-label">Target Release</span>
          <span class="parity-meta-val mono">{}</span>
        </div>
        <div class="parity-meta-item">
          <span class="parity-meta-label">Verified Test Suites</span>
          <span class="parity-meta-val mono">{} in-game scenarios</span>
        </div>
        <div class="parity-meta-item">
          <span class="parity-meta-label">Automated Sync</span>
          <span class="parity-meta-val mono">{}</span>
        </div>
      </div>
      <div class="parity-meta-msg mono term-dim">
        &gt; git log -1: {} ({})
      </div>
    </div>

    <div class="notice-box" style="margin-bottom: 2rem;">
      <span class="notice-badge">Transparent Parity</span>
      <div style="font-size: 0.9rem; color: var(--text-muted); line-height: 1.7;">
        <strong>Granular Parity Matrix:</strong> We don&rsquo;t hide missing mobs or partial mechanics. Every single mob, block, item, and system is cataloged as <span class="badge-tag badge-green" style="padding: 1px 6px; font-size: 0.72rem;">Ready</span>, <span class="badge-tag badge-rust" style="padding: 1px 6px; font-size: 0.72rem;">In Progress</span>, or <span class="badge-tag badge-gray" style="padding: 1px 6px; font-size: 0.72rem;">Planned</span>. Verified entries link to real in-game automated test scripts (<code class="mono">tests/ingame/</code>). The entire catalog lives in <a href="https://github.com/PommeMC/Client" target="_blank" rel="noreferrer"><code class="mono">data/parity.json</code></a> and syncs directly from upstream.
      </div>
    </div>

    <div class="parity-toolbar">
      <div class="parity-toolbar-main">
        <div class="tracker-search-wrap">
          <span class="tracker-search-icon mono" aria-hidden="true">&gt;_</span>
          <input type="text" id="trackerSearch" class="tracker-search-input" placeholder="Search mobs, blocks, mechanics, test scripts..." aria-label="Search mobs, blocks, mechanics, or keywords">
          <kbd class="mono">/</kbd>
        </div>
        <div class="parity-progress" role="img" aria-label="Engine implementation progress: {} ready, {} in progress, {} planned">
          <div class="tracker-bar-track">
            <div class="tracker-bar-fill-ready" style="width: {}%;"></div>
            <div class="tracker-bar-fill-progress" style="width: {}%;"></div>
            <div class="tracker-bar-fill-planned" style="width: {}%;"></div>
          </div>
          <div class="parity-progress-legend mono">
            <span><i class="dot dot-ready"></i>{} Ready</span>
            <span><i class="dot dot-progress"></i>{} WIP</span>
            <span><i class="dot dot-planned"></i>{} Planned</span>
          </div>
        </div>
      </div>

      <div class="parity-pills-row">
        <div class="tracker-pills" role="group" aria-label="Filter by category">
          {}
        </div>
        <div class="tracker-pills" role="group" aria-label="Filter by status">
          {}
        </div>
        <div class="parity-count mono">
          Showing <span id="trackerCount">{}</span>/{} entries
        </div>
      </div>
    </div>

    <div class="tracker-groups">
      {}
    </div>

    <div class="section-head" style="margin-top: 5rem;">
      <div class="section-kicker">Automated Verification</div>
      <h2 class="section-title">Differential & In-Game <span class="badge-highlight">Test Harness</span></h2>
      <p class="section-subtitle">
        Scripted end-to-end runs execute real client builds against vanilla Mojang 26.2 servers across 88 in-game scenarios to guarantee gameplay-level parity and prevent regressions.
      </p>
    </div>

    <div class="verification-log">
      <div class="verification-body">
        {}
      </div>
      <div class="verification-footer mono">
        <span>Suite: tools/ingame-test against vanilla 26.2 server</span>
        <span>88 automated toml scenarios</span>
      </div>
    </div>
  </div>
</section>"#,
        total,
        domain_cards_html,
        meta.commit,
        meta.commit,
        meta.tag,
        meta.test_count,
        meta.updated_at,
        meta.commit_title,
        meta.commit_date,
        ready,
        in_progress,
        planned,
        ready_pct,
        progress_pct,
        planned_pct,
        ready,
        in_progress,
        planned,
        cat_pills_html,
        status_pills_html,
        total,
        total,
        groups_html,
        harness_rows
    )
}

fn build_architecture() -> String {
    r##"<section class="section" style="padding-top: 4.5rem; padding-bottom: 2rem;">
  <div class="container">
    <div class="section-head text-center" style="max-width: 860px; margin-left: auto; margin-right: auto; text-align: center;">
      <div class="section-kicker">Systems Architecture</div>
      <h1 class="section-title" style="font-size: clamp(2.4rem, 5vw, 3.4rem); margin-bottom: 1rem;">
        Engineered in <span class="badge-highlight">Rust</span> from day one
      </h1>
      <p class="section-subtitle" style="margin-left: auto; margin-right: auto; font-size: 1.15rem; line-height: 1.65;">
        No JVM runtime. No separate launcher. How Pomme decomposes a Minecraft client into audit-friendly, zero-allocation subsystems.
      </p>
    </div>

    <div class="brutal-grid" style="margin-top: 3rem;">
      <div class="brutal-card">
        <span class="brutal-metric">&lt; 300 MB</span>
        <p>Resident RAM at 16 chunks. <span>Vanilla Java requires 2,048 &ndash; 4,096 MB before texture caching.</span></p>
      </div>
      <div class="brutal-card">
        <span class="brutal-metric">0 ms GC</span>
        <p>Deterministic memory management. <span>Zero stuttering pauses from Java garbage collector thread sweeps.</span></p>
      </div>
      <div class="brutal-card">
        <span class="brutal-metric">1.4s Boot</span>
        <p>Cold launch to menu. <span>Zero JVM classloading or bytecode warmup delay.</span></p>
      </div>
      <div class="brutal-card">
        <span class="brutal-metric">GPL-3.0</span>
        <p>100% Free and Open Source. <span>Zero Mojang code, no proprietary telemetry, fully auditable.</span></p>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Execution Pipeline</div>
      <h2 class="section-title">Multithreaded <span class="badge-highlight">Game Loop &amp; Tick Model</span></h2>
      <p class="section-subtitle">
        Pomme separates frame presentation, fixed 20 Hz simulation ticks, and asynchronous I/O into dedicated worker threads with zero lock contention.
      </p>
    </div>

    <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 1.5rem; margin-top: 2rem;">
      <div class="guide-step">
        <span class="guide-num mono">01</span>
        <div>
          <h4>Main Thread &bull; Windowing &amp; Input</h4>
          <p>
            Handles OS event polling via <code class="mono">winit</code>, raw mouse input curves, keyboard rebinding, and immediate UI frame submission. Never blocks on network packets, chunk meshing, or disk operations.
          </p>
        </div>
      </div>

      <div class="guide-step">
        <span class="guide-num mono">02</span>
        <div>
          <h4>Simulation Thread &bull; Fixed 20 Hz Loop</h4>
          <p>
            Runs a deterministic 50 ms tick loop decoupled from display refresh rates. Evaluates player physics, collision bounding boxes, entity animation matrices, and sends input packets to the server.
          </p>
        </div>
      </div>

      <div class="guide-step">
        <span class="guide-num mono">03</span>
        <div>
          <h4>Rayon Worker Pool &bull; Chunk Decompression</h4>
          <p>
            Worker threads dynamically decompress network packets, parse paletted block states, execute ambient occlusion interpolation, and generate 32-byte packed vertex quad buffers in parallel across all CPU cores.
          </p>
        </div>
      </div>

      <div class="guide-step">
        <span class="guide-num mono">04</span>
        <div>
          <h4>GPU Transfer &bull; Vulkan Queue Submission</h4>
          <p>
            Dedicated Vulkan transfer and graphics queue families. Submits pre-recorded SPIR-V command buffers to Vulkan 1.3 or Apple Metal (MoltenVK) synchronized with display VSync without pipeline stall bubbles.
          </p>
        </div>
      </div>
    </div>

    <div class="terminal-card" style="margin-top: 2rem;">
      <div class="terminal-header">
        <div class="terminal-dots"><div class="terminal-dot"></div><div class="terminal-dot"></div><div class="terminal-dot"></div></div>
        <div class="terminal-title">pomme-bench --threading</div>
        <button class="term-copy-btn" data-copy="pomme-client --bench-threads">Copy</button>
      </div>
      <div class="terminal-body">
        <div><span class="term-cmd">$</span> pomme-client --bench-threads</div>
        <div class="term-dim">[threading] CPU topology detected: 16 logical cores, 8 performance / 8 efficiency</div>
        <div><span class="term-rust">[simulation]</span> <span class="term-highlight">Tick duration: 50.00 ms &plusmn; 0.08 ms (20.00 TPS target locked)</span></div>
        <div><span class="term-rust">[rayon-pool]</span> <span class="term-dim">Active worker threads: 14 &bull; Task stealing latency: 0.002 ms</span></div>
        <div><span class="term-cmd">[gpu-pipeline]</span> <span class="term-highlight">Frame pacing: 165 FPS &bull; Frame time variance: &lt; 0.2 ms</span></div>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Memory Architecture</div>
      <h2 class="section-title">Deterministic Arenas vs <span class="badge-highlight">Java GC Pauses</span></h2>
      <p class="section-subtitle">
        Why Minecraft Java Edition suffers from random micro-stutters and how Pomme achieves zero garbage collector pauses.
      </p>
    </div>

    <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 2rem; margin-top: 2rem;">
      <div style="background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 2rem;">
        <span class="badge-tag badge-rust" style="margin-bottom: 1rem; display: inline-block;">Java HotSpot VM</span>
        <h3 style="font-size: 1.3rem; font-weight: 800; margin-bottom: 0.75rem;">Generational GC Heap Fragmentation</h3>
        <p style="color: var(--text-muted); font-size: 0.9rem; line-height: 1.65; margin-bottom: 1.25rem;">
          Java creates millions of short-lived objects every second (<code class="mono">BlockPos</code>, <code class="mono">Vec3d</code>, byte arrays). When young generations fill up, JVM Stop-The-World pauses freeze rendering for 15 to 120 ms while collecting dead objects.
        </p>
        <ul style="list-style: none; display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.85rem; color: var(--text-main);">
          <li>&bull; Sawtooth memory curve with random micro-stutters</li>
          <li>&bull; Resident heap footprint between 2,048 MB and 4,096 MB</li>
          <li>&bull; Complex JVM tuning flags required (G1GC, ZGC, Shenandoah)</li>
        </ul>
      </div>

      <div style="background: var(--bg-panel); border: 2px solid var(--accent-green); border-radius: 8px; padding: 2rem; box-shadow: 4px 4px 0px rgba(46, 204, 113, 0.2);">
        <span class="badge-tag badge-green" style="margin-bottom: 1rem; display: inline-block;">Pomme Architecture</span>
        <h3 style="font-size: 1.3rem; font-weight: 800; margin-bottom: 0.75rem; color: var(--accent-green);">Deterministic Slot Arenas</h3>
        <p style="color: var(--text-muted); font-size: 0.9rem; line-height: 1.65; margin-bottom: 1.25rem;">
          Pomme manages chunk vertex buffers and section palettes in contiguous, fixed-size arena memory pools. When chunks unload, their slots are instantly recycled into the free list in O(1) time without triggering garbage sweeps.
        </p>
        <ul style="list-style: none; display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.85rem; color: var(--text-main);">
          <li>&bull; <strong>0.000 ms GC pause latency:</strong> display VSync dictates frame pacing</li>
          <li>&bull; <strong>Flat memory curve:</strong> &lt; 300 MB resident footprint at 16 chunks</li>
          <li>&bull; Predictable memory reuse during high-speed elytra flight</li>
        </ul>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Technical Comparison</div>
      <h2 class="section-title">Architectural Decisions: <span class="badge-highlight">Pomme vs Vanilla</span></h2>
      <p class="section-subtitle">
        A side-by-side engineering breakdown between native Rust systems programming and Java HotSpot VM.
      </p>
    </div>

    <div class="table-container">
      <table class="specs-table">
        <thead>
          <tr>
            <th style="width: 25%;">Architecture Metric</th>
            <th style="width: 37%;">Pomme (Rust &amp; Vulkan)</th>
            <th style="width: 38%;">Vanilla (Java Edition &amp; JVM)</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Runtime Model</td>
            <td style="color: var(--accent-green); font-weight: 600;">Single native AOT binary compiled to machine code</td>
            <td style="color: var(--text-muted);">Java HotSpot VM (bytecode JIT compilation &amp; warmup)</td>
          </tr>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Graphics Backend</td>
            <td style="color: var(--accent-green); font-weight: 600;">Direct Vulkan 1.3 &amp; Apple Metal (SPIR-V pipelines)</td>
            <td style="color: var(--text-muted);">Legacy OpenGL 3.2 compatibility profile state machine</td>
          </tr>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Garbage Collection</td>
            <td style="color: var(--accent-green); font-weight: 600;">0 ms (Deterministic contiguous memory arenas)</td>
            <td style="color: var(--text-muted);">Stop-the-world GC sweeps (random frame micro-stutters)</td>
          </tr>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Singleplayer Server</td>
            <td style="color: var(--accent-green); font-weight: 600;">In-process multithreaded SteelMC (zero-copy memory channel)</td>
            <td style="color: var(--text-muted);">Secondary local JVM server instance on localhost TCP loopback</td>
          </tr>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Wire Protocol Codecs</td>
            <td style="color: var(--accent-green); font-weight: 600;">Compile-time static dispatch tables (zero allocation)</td>
            <td style="color: var(--text-muted);">Runtime reflection, dynamic class loading &amp; Netty ByteBuf overhead</td>
          </tr>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Resident Memory</td>
            <td style="color: var(--accent-green); font-weight: 600;">&lt; 300 MB at 16 chunks render distance</td>
            <td style="color: var(--text-muted);">2,048 &ndash; 4,096 MB before texture caching</td>
          </tr>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Cold Boot Latency</td>
            <td style="color: var(--accent-green); font-weight: 600;">~1.4 seconds from launch to interactive main menu</td>
            <td style="color: var(--text-muted);">11.8 &ndash; 14.5 seconds (JVM warmup and classloading)</td>
          </tr>
          <tr>
            <td class="mono" style="color: var(--text-main); font-weight: 700;">Multi-Platform</td>
            <td style="color: var(--accent-green); font-weight: 600;">Unified Rust codebase for Windows, Linux, macOS, Android, iOS</td>
            <td style="color: var(--text-muted);">Separate Java Edition desktop &amp; C++ Bedrock mobile engines</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Workspace Structure</div>
      <h2 class="section-title">Workspace <span class="badge-highlight">Crate Hierarchy</span></h2>
      <p class="section-subtitle">
        Pomme separates concerns into decoupled, reusable Rust crates. Every crate is independently auditable and verifiable.
      </p>
    </div>

    <div class="benchmarks-grid">
      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title mono" style="color: var(--accent-rust);">pomme-client</div>
          <span class="badge-tag badge-green">Core Engine</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6;">
          Coordinates main game loop, windowing via winit, player physics, UI/HUD screens, Anvil chunk streaming, and delegates rendering to pomme-gpu.
        </p>
      </div>

      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title mono" style="color: var(--accent-rust);">pomme-protocol</div>
          <span class="badge-tag badge-green">Networking</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6;">
          Decodes and serializes packets across Minecraft 1.20 to 26.3 snapshots with automated packet rewrite dispatch tables generated at compile-time.
        </p>
      </div>

      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title mono" style="color: var(--accent-rust);">pomme-singleplayer</div>
          <span class="badge-tag badge-green">Integrated Server</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6;">
          In-process multithreaded SteelMC server integration for instant offline worlds and standard vanilla Anvil .mca region chunk save persistence.
        </p>
      </div>

      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title mono" style="color: var(--accent-rust);">pomme-gpu-allocator</div>
          <span class="badge-tag badge-green">Vulkan Memory</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6;">
          Dedicated Vulkan video memory allocator managing contiguous vertex and index buffer pools with zero driver stall bubbles.
        </p>
      </div>

      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title mono" style="color: var(--accent-rust);">pomme-auth &amp; install</div>
          <span class="badge-tag badge-green">Auth &amp; Assets</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6;">
          Direct Microsoft OAuth2 exchange with OS Keyring token storage. Hash-verified asset streaming from Mojang CDN and in-app self-updating.
        </p>
      </div>

      <div class="bench-card">
        <div class="bench-header">
          <div class="bench-title mono" style="color: var(--accent-rust);">pomme-android &amp; ios</div>
          <span class="badge-tag badge-rust">Mobile Ports</span>
        </div>
        <p style="color: var(--text-muted); font-size: 0.85rem; line-height: 1.6;">
          NativeActivity entry point for Android arm64 APK and Xcode Apple Metal project for iOS with virtual on-screen touch controls.
        </p>
      </div>
    </div>

    <div class="notice-box" style="margin-top: 3.5rem;">
      <span class="notice-badge">Verification</span>
      <div style="font-size: 0.9rem; color: var(--text-muted); line-height: 1.7;">
        Pomme enforces architectural correctness through scripted in-game differential test suites in <code class="mono" style="color: var(--accent-rust);">tests/ingame/</code>. Automated headless clients join a live vanilla 26.2 Mojang server to assert packet formats, inventory transitions, and visual geometry match vanilla byte-for-byte.
      </div>
    </div>
  </div>
</section>"##.to_string()
}

fn build_community(contributors: &[Contributor], repo: &GithubRepo) -> String {
    let humans: Vec<&Contributor> = contributors
        .iter()
        .filter(|c| !c.login.contains("[bot]"))
        .collect();

    let total_contributors = humans.len();
    let total_commits: u64 = humans.iter().map(|c| c.contributions).sum();
    let max_commits = humans.first().map(|c| c.contributions).unwrap_or(1);

    let mut most_commits_html = String::new();
    for (idx, c) in humans.iter().take(6).enumerate() {
        let rank = idx + 1;
        let pct = (c.contributions as f64 / max_commits as f64 * 100.0).max(3.0);
        most_commits_html.push_str(&format!(
            r#"<li class="most-commit-item">
  <span class="commit-rank-num">{}</span>
  <img src="{}" alt="{}" class="commit-rank-avatar" width="48" height="48">
  <div class="commit-rank-info">
    <a href="{}" target="_blank" rel="noreferrer" class="commit-rank-name">{}</a>
    <div class="commit-rank-bar">
      <div class="commit-rank-fill" style="width: {:.1}%;"></div>
    </div>
  </div>
  <span class="commit-rank-count"><strong><span data-counter="{}">{}</span></strong> commits</span>
</li>"#,
            rank, c.avatar_url, c.login, c.html_url, c.login, pct, c.contributions, c.contributions
        ));
    }

    let mut everyone_html = String::new();
    for c in &humans {
        everyone_html.push_str(&format!(
            r#"<li>
  <a href="{}" target="_blank" rel="noreferrer" class="everyone-avatar-card" title="{}, {} commits" aria-label="{}, {} commits">
    <img src="{}" alt="{}" loading="lazy" width="64" height="64">
  </a>
</li>"#,
            c.html_url, c.login, c.contributions, c.login, c.contributions, c.avatar_url, c.login
        ));
    }

    format!(
        r#"<section class="section" style="padding-top: 4.5rem; padding-bottom: 2.5rem;">
  <div class="container">
    <div class="section-kicker">Open Source Community</div>
    <h1 class="hero-title" style="font-size: clamp(2.5rem, 5vw, 4rem); margin-bottom: 1rem;">
      Built by its contributors<span style="color: var(--accent-green);">.</span>
    </h1>
    <p class="hero-desc" style="max-width: 650px; margin-bottom: 0;">
      Pomme is built by an amazing community of open source contributors. Every commit below is someone's spare time.
    </p>

    <div class="contributors-stats-bar">
      <div class="contributor-stat-item">
        <span class="stat-huge" data-counter="{}">{}</span>
        <span class="stat-unit">contributors</span>
      </div>
      <div class="contributor-stat-item">
        <span class="stat-huge" data-counter="{}">{}</span>
        <span class="stat-unit">commits</span>
      </div>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="contributors-split-grid">
      <div class="most-commits-col">
        <h2 class="section-title" style="font-size: 1.75rem; margin-bottom: 1.5rem;">Most commits</h2>
        <ol class="most-commits-list">
          {}
        </ol>
      </div>

      <div class="everyone-else-col">
        <h2 class="section-title" style="font-size: 1.75rem; margin-bottom: 1.5rem;">Everyone else</h2>
        <ul class="everyone-grid">
          {}
        </ul>
      </div>
    </div>
  </div>
</section>

<section class="contribute-callout">
  <div class="container contribute-callout-inner">
    <div>
      <h2 style="font-size: 1.85rem; font-weight: 800; margin-bottom: 0.4rem;">Want to contribute?</h2>
      <p style="color: var(--text-muted); font-size: 1.05rem;">Join our community and help build the fastest Minecraft client ever.</p>
    </div>
    <div class="contribute-links">
      <a href="https://github.com/PommeMC/Client#contributing" target="_blank" rel="noreferrer" class="contribute-link">Contributing guide &rarr;</a>
      <a href="https://github.com/PommeMC/Client" target="_blank" rel="noreferrer" class="contribute-link">View on GitHub &rarr;</a>
      <a href="https://discord.gg/ucBA55bHPR" target="_blank" rel="noreferrer" class="contribute-link">Join Discord &rarr;</a>
    </div>
  </div>
</section>

<section class="section">
  <div class="container">
    <div class="section-head">
      <div class="section-kicker">Community Hubs</div>
      <h2 class="section-title">Where to find us</h2>
      <p class="section-subtitle">Connect with developers, report bugs, test bleeding-edge builds, or sponsor client infrastructure.</p>
    </div>

    <div class="community-grid-hub">
      <div class="community-hub-card">
        <div class="community-hub-icon">DISCORD</div>
        <h3 style="font-size: 1.35rem; font-weight: 800; margin-bottom: 0.5rem;">Developer Chat</h3>
        <p style="color: var(--text-muted); font-size: 0.9rem; line-height: 1.6; margin-bottom: 1.75rem;">
          Connect directly with the development team, follow live Git commit feeds, participate in engine discussions, and test bleeding-edge builds.
        </p>
        <a href="https://discord.gg/ucBA55bHPR" class="btn btn-primary" target="_blank" rel="noreferrer" style="width: 100%;">
          Join Discord Community &rarr;
        </a>
      </div>

      <div class="community-hub-card">
        <div class="community-hub-icon" style="color: var(--accent-rust);">GITHUB</div>
        <h3 style="font-size: 1.35rem; font-weight: 800; margin-bottom: 0.5rem;">Source Code &amp; PRs</h3>
        <p style="color: var(--text-muted); font-size: 0.9rem; line-height: 1.6; margin-bottom: 1.75rem;">
          Explore all crates, report renderer bugs, and submit pull requests. Currently at {} stars and {} community forks.
        </p>
        <a href="https://github.com/PommeMC/Client" class="btn btn-secondary mono" target="_blank" rel="noreferrer" style="width: 100%;">
          View PommeMC/Client &rarr;
        </a>
      </div>

      <div class="community-hub-card">
        <div class="community-hub-icon" style="color: var(--accent-cyan);">SPONSORS</div>
        <h3 style="font-size: 1.35rem; font-weight: 800; margin-bottom: 0.5rem;">Support Development</h3>
        <p style="color: var(--text-muted); font-size: 0.9rem; line-height: 1.6; margin-bottom: 1.75rem;">
          Support the maintainers directly through GitHub Sponsors to fund dedicated build machines, iOS developer accounts, and test infrastructure.
        </p>
        <a href="https://github.com/sponsors/Purdze" class="btn btn-secondary" target="_blank" rel="noreferrer" style="width: 100%;">
          Sponsor on GitHub &rarr;
        </a>
      </div>
    </div>
  </div>
</section>"#,
        total_contributors,
        total_contributors,
        total_commits,
        total_commits,
        most_commits_html,
        everyone_html,
        repo.stargazers_count,
        repo.forks_count
    )
}

fn write_meta_files(out_dir: &Path) {
    let pages = [
        ("", "1.0"),
        ("downloads", "1.0"),
        ("parity", "0.9"),
        ("architecture", "0.8"),
        ("community", "0.7"),
    ];

    let mut sitemap = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );
    for (page, priority) in pages {
        let loc = if page.is_empty() {
            format!("{}/", SITE_URL)
        } else {
            format!("{}/{}/", SITE_URL, page)
        };
        sitemap.push_str(&format!(
            r#"  <url><loc>{}</loc><changefreq>weekly</changefreq><priority>{}</priority></url>
"#,
            loc, priority
        ));
    }
    sitemap.push_str("</urlset>\n");

    let robots = format!(
        "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
        SITE_URL
    );

    let headers = r#"/*
  Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' https://fonts.googleapis.com 'unsafe-inline'; font-src 'self' https://fonts.gstatic.com; img-src 'self' https://avatars.githubusercontent.com data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; object-src 'none'
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: camera=(), microphone=(), geolocation=()
/assets/*
  Cache-Control: public, max-age=86400, stale-while-revalidate=604800
/favicon.ico
  Cache-Control: public, max-age=604800
"#;

    let redirects = r#"/downloads.html /downloads/ 301
/parity.html /parity/ 301
/architecture.html /architecture/ 301
/community.html /community/ 301
/contributors.html /contributors/ 301
/download /downloads/ 301
"#;

    let _ = fs::write(out_dir.join("sitemap.xml"), sitemap);
    let _ = fs::write(out_dir.join("robots.txt"), robots);
    let _ = fs::write(out_dir.join("_headers"), headers);
    let _ = fs::write(out_dir.join("_redirects"), redirects);

    let well_known_dir = out_dir.join(".well-known");
    let _ = fs::create_dir_all(&well_known_dir);
    let ms_ident =
        r#"{"associatedApplications":[{"applicationId":"cb8a104f-4d95-4d71-a3db-f2ba6c326c97"}]}"#;
    let _ = fs::write(
        well_known_dir.join("microsoft-identity-association.json"),
        ms_ident,
    );
    let _ = fs::write(out_dir.join(".nojekyll"), "");
}

fn build_404() -> String {
    format!(
        r#"<section class="section section-404">
  <div class="container" style="max-width: 860px; text-align: center;">
    <div class="hero-label" style="display: inline-flex; margin-bottom: 1.25rem;">
      <span class="pulse-dot"></span>
      <span>ERROR 404</span>
      <span>&bull;</span>
      <span>CHUNK_NOT_FOUND</span>
    </div>
    <h1 class="hero-title" style="margin-bottom: 1rem;">Coordinate <span class="badge-highlight">out of bounds</span></h1>
    <p class="hero-desc" style="max-width: 620px; margin: 0 auto 2rem auto;">
      The requested route or chunk does not exist in the world save or navigation graph.
    </p>

    <div class="hero-ctas" style="justify-content: center; margin-bottom: 3rem;">
      <a href="/" class="btn btn-primary btn-lg">Return to Spawn</a>
      <a href="/downloads/" class="btn btn-secondary btn-lg">Download Client</a>
    </div>

    <div class="terminal-card" style="text-align: left; margin: 0 auto 3.5rem auto;">
      <div class="terminal-header">
        <div class="terminal-dots">
          <div class="terminal-dot"></div>
          <div class="terminal-dot"></div>
          <div class="terminal-dot"></div>
        </div>
        <div class="terminal-title">pomme-client ~ route resolution trace</div>
      </div>
      <div class="terminal-body">
        <div><span class="term-cmd">$</span> pomme-client --navigate {}/unknown_chunk</div>
        <div class="term-dim">[pomme-route] Resolving client URL route...</div>
        <div class="term-comment">[pomme-world] Error: Coordinate unmapped (HTTP 404: chunk missing)</div>
        <div class="term-dim">[pomme-gpu] Pipeline flushed &bull; Viewport idle &bull; Ready for input</div>
        <div><span class="term-highlight">&gt; Fallback navigation ready &mdash; select a verified destination below:</span><span class="term-cursor"></span></div>
      </div>
    </div>

    <div class="error-destinations">
      <a href="/" class="error-dest-card">
        <span class="error-dest-kicker mono">01 // HOME</span>
        <h3 class="error-dest-title">Overview</h3>
        <p class="error-dest-desc">Vulkan 1.3 presentation, 165 FPS frame timing, and cold-start benchmark race.</p>
      </a>
      <a href="/downloads/" class="error-dest-card">
        <span class="error-dest-kicker mono">02 // BINARIES</span>
        <h3 class="error-dest-title">Downloads</h3>
        <p class="error-dest-desc">Standalone native executables for Linux (AppImage), Windows (x64), and macOS.</p>
      </a>
      <a href="/parity/" class="error-dest-card">
        <span class="error-dest-kicker mono">03 // STATUS</span>
        <h3 class="error-dest-title">Vanilla Parity</h3>
        <p class="error-dest-desc">Exhaustive tracker of all mobs, blocks, redstone, and mechanics against vanilla MC.</p>
      </a>
      <a href="/architecture/" class="error-dest-card">
        <span class="error-dest-kicker mono">04 // ARCH</span>
        <h3 class="error-dest-title">Architecture</h3>
        <p class="error-dest-desc">Audit-friendly Rust crates: pomme-gpu, pomme-protocol, and in-process SteelMC.</p>
      </a>
    </div>
  </div>
</section>"#,
        SITE_URL
    )
}

fn apply_base_path(html: &str, base: &str) -> String {
    if base.is_empty() {
        return html.to_string();
    }
    html.replace(r#"href="/"#, &format!(r#"href="{}/"#, base))
        .replace(r#"src="/"#, &format!(r#"src="{}/"#, base))
        .replace(r#"poster="/"#, &format!(r#"poster="{}/"#, base))
        .replace(
            r#"content="/assets/"#,
            &format!(r#"content="{}/assets/"#, base),
        )
}

fn build_site() {
    let base_dir = resolve_base_dir();
    let out_dir = base_dir.join("dist");
    fs::create_dir_all(&out_dir).expect("failed to create dist dir");
    fs::create_dir_all(out_dir.join("assets")).expect("failed to create dist/assets");

    println!("Fetching real project data from GitHub API...");
    let (repo, release, checksums, contributors, commits) = fetch_real_data();
    println!(
        "Fetched data for release: {}, stars: {}, recent commits: {}, contributors: {}",
        release.tag_name,
        repo.stargazers_count,
        commits.len(),
        contributors.len()
    );

    let version = release
        .tag_name
        .trim_start_matches("client-")
        .trim_start_matches('v')
        .split('+')
        .next()
        .unwrap_or(&release.tag_name)
        .to_string();

    let index_page = render_layout(
        "Home",
        "High-performance Minecraft client written in Rust. No Java, one binary to run, desktop, Android and iOS on the same codebase.",
        "/",
        &version,
        &build_index(&repo, &commits, &version),
    );
    let downloads_page = render_layout(
        "Downloads",
        "Download the Pomme client for Windows, Linux and macOS. One standalone native binary: no launcher, no JVM.",
        "/downloads",
        &version,
        &build_downloads(&release, &checksums),
    );
    let parity_page = render_layout(
        "Vanilla Parity",
        "Track which Pomme features and vanilla Minecraft mechanics are implemented, in progress or planned.",
        "/parity",
        &version,
        &build_parity(),
    );
    let arch_page = render_layout(
        "Architecture",
        "How the Pomme client organizes its Rust crates, memory model and rendering pipeline without JVM overhead.",
        "/architecture",
        &version,
        &build_architecture(),
    );
    let comm_page = render_layout(
        "Contributors",
        "The open source community behind the Pomme client: contributors, commit stats and community hubs.",
        "/community",
        &version,
        &build_community(&contributors, &repo),
    );
    let not_found_page = render_layout(
        "404 — Chunk not found",
        "This page failed to mesh: chunk not found. Mine an unrendered chunk on the Pomme 404 page to find a diamond and the way home.",
        "/404",
        &version,
        &build_404(),
    );

    let base = std::env::var("BASE_PATH")
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();

    fs::write(
        out_dir.join("index.html"),
        apply_base_path(&index_page, &base),
    )
    .expect("failed to write index.html");
    fs::write(
        out_dir.join("404.html"),
        apply_base_path(&not_found_page, &base),
    )
    .expect("failed to write 404.html");

    for (name, content) in [
        ("downloads", downloads_page),
        ("parity", parity_page),
        ("architecture", arch_page),
        ("community", comm_page.clone()),
        ("contributors", comm_page),
    ] {
        let dir = out_dir.join(name);
        fs::create_dir_all(&dir).expect("failed to create page directory");
        fs::write(dir.join("index.html"), apply_base_path(&content, &base))
            .expect("failed to write page");
    }

    write_meta_files(&out_dir);

    let _ = copy_dir_all(base_dir.join("assets"), out_dir.join("assets"));
    let _ = fs::copy(base_dir.join("favicon.ico"), out_dir.join("favicon.ico"));

    println!("Static site generated and assets synced successfully in dist/!");
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn resolve_page(root: &Path, url_path: &str) -> (std::path::PathBuf, u16) {
    let rel = url_path.trim().trim_start_matches('/');
    let rel = percent_decode(rel);
    if rel.split('/').any(|seg| seg == "..") {
        return (root.join("404.html"), 404);
    }

    let candidate = if rel.is_empty() {
        root.join("index.html")
    } else {
        root.join(&rel)
    };

    if candidate.is_dir() {
        return (candidate.join("index.html"), 200);
    }
    if candidate.is_file() {
        return (candidate, 200);
    }

    if !rel.contains('.') {
        if let Some(known) = candidate.to_str() {
            let as_dir = format!("{}.html", known);
            let as_dir = Path::new(&as_dir);
            if as_dir.is_file() {
                return (as_dir.to_path_buf(), 200);
            }
            let as_page = root.join(format!("{}.html", rel));
            if as_page.is_file() {
                return (as_page, 200);
            }
            let index = candidate.join("index.html");
            if index.is_file() {
                return (index, 200);
            }
        }
    }

    (root.join("404.html"), 404)
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "xml" => "text/xml",
        "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn run_server(port: u16) -> std::io::Result<()> {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let base_dir = resolve_base_dir();
    let root =
        fs::canonicalize(base_dir.join("dist")).expect("dist/ not found — run a build first");
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!(
        "Serving dist/ at http://localhost:{} (Ctrl+C to stop)",
        port
    );

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else {
            continue;
        };
        let root = root.clone();
        std::thread::spawn(move || {
            let mut buf = [0_u8; 2048];
            let len = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..len]);
            let Some(request_line) = request.lines().next() else {
                return;
            };
            let mut parts = request_line.split_whitespace();
            let method = parts.next().unwrap_or("");
            let target = parts.next().unwrap_or("/");
            let url_path = target.split('?').next().unwrap_or(target);

            if method != "GET" && method != "HEAD" {
                let body = "Method not supported\r\n";
                let _ = stream.write_all(
                    format!("HTTP/1.1 405 METHOD NOT ALLOWED\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).as_bytes(),
                );
                return;
            }

            let (file_path, status) = resolve_page(&root, url_path);
            match fs::read(&file_path) {
                Ok(body) => {
                    let status_line = match status {
                        200 => "HTTP/1.1 200 OK",
                        _ => "HTTP/1.1 404 NOT FOUND",
                    };
                    let head = format!(
                        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
                        status_line,
                        content_type(&file_path),
                        body.len()
                    );
                    if stream.write_all(head.as_bytes()).is_ok() && method == "GET" {
                        let _ = stream.write_all(&body);
                    }
                }
                Err(_) => {
                    let body = "dist file missing\r\n";
                    let _ = stream.write_all(
                        format!("HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).as_bytes(),
                    );
                }
            }
        });
    }
    Ok(())
}

fn main() {
    let mut serve_mode = false;
    let mut port: u16 = 4321;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "serve" | "--serve" => serve_mode = true,
            "--port" | "-p" => {
                port = args.next().and_then(|v| v.parse().ok()).unwrap_or(4321_u16);
            }
            _ => {}
        }
    }

    build_site();

    if serve_mode {
        if let Err(err) = run_server(port) {
            eprintln!("server failed: {}", err);
            std::process::exit(1);
        }
    }
}
