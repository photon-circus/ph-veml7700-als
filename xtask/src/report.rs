pub struct Reporter {
    step_number: u32,
    skipped: u32,
    current_step: String,
}

impl Reporter {
    pub fn new() -> Self {
        Self {
            step_number: 0,
            skipped: 0,
            current_step: String::from("start"),
        }
    }

    pub fn begin(&mut self, title: &str) {
        self.step_number += 1;
        self.current_step = title.to_string();
        println!("\n[ci {:02}] {title}", self.step_number);
    }

    pub fn skip(&mut self, note: &str) {
        self.skipped += 1;
        println!("        SKIP: {note}");
    }

    pub fn note(&self, msg: &str) {
        println!("        {msg}");
    }

    pub fn fail(&self) {
        eprintln!(
            "\n[ci] FAIL at step {}: {}",
            self.step_number, self.current_step
        );
    }

    pub fn step_number(&self) -> u32 {
        self.step_number
    }

    pub fn skipped(&self) -> u32 {
        self.skipped
    }
}

pub fn print_footer(
    profile: &str,
    only: Option<&str>,
    steps: u32,
    skipped: u32,
    evidence_path: Option<&str>,
    release_commit: Option<&str>,
    package_sha: Option<&str>,
) {
    // A single-step run carries the profile name but establishes almost none of
    // what that profile means. Label it so a pasted footer cannot be read as the
    // authoritative gate.
    match only {
        Some(id) => {
            println!("\n[ci] PASS ({profile} --only {id}): {steps} steps, {skipped} skipped.");
            println!("[ci] This ran one selected step, not the {profile} gate. It establishes");
            println!("[ci] that step alone; the complete run remains authoritative.");
        }
        None => println!("\n[ci] PASS ({profile}): {steps} steps, {skipped} skipped."),
    }
    if only.is_none() && profile == "bounded" {
        println!("[ci] This is a partial gate. It covers only part of the release gate;");
        println!("[ci] the full local run remains authoritative.");
    }
    if profile == "release" {
        if let (Some(path), Some(commit), Some(sha)) = (evidence_path, release_commit, package_sha)
        {
            println!("[ci] Release evidence: {path}");
            println!("[ci] Commit {commit}, archive SHA-256 {sha}");
        }
        println!("[ci] No registry action was performed: no publish, tag, release, or");
        println!("[ci] credential use. Publication remains a separate recorded decision.");
    }
    println!("[ci] This gate establishes the implemented host boundary only. It does");
    println!("[ci] not establish physical-device or calibrated-optical behavior.");
}
