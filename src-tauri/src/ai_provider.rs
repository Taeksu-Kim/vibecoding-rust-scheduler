use serde::{Deserialize, Serialize};
use std::process::Command as StdCommand;
use std::path::PathBuf;
use serde_json;

/// AI 프로바이더 종류
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiProvider {
    /// Claude Code CLI
    Claude,
    /// GitHub Copilot CLI
    Copilot,
}

impl Default for AiProvider {
    fn default() -> Self {
        AiProvider::Claude
    }
}

/// AI 프로바이더별 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: AiProvider,
    /// Claude Code CLI 경로 (옵션)
    pub claude_path: Option<String>,
    /// Copilot CLI 경로 (옵션)
    pub copilot_path: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Claude,
            claude_path: None,
            copilot_path: None,
        }
    }
}

impl AiConfig {
    /// CLI 경로 자동 탐지
    pub fn detect_cli_path(provider: &AiProvider) -> Option<PathBuf> {
        let mut potential_paths: Vec<PathBuf> = Vec::new();

        match provider {
            AiProvider::Claude => {
                // Try environment variable (Windows)
                if let Ok(appdata) = std::env::var("APPDATA") {
                    potential_paths.push(PathBuf::from(appdata).join(r"npm\node_modules\@anthropic-ai\claude-code\cli.js"));
                }

                // Unix-like systems
                if let Ok(home) = std::env::var("HOME") {
                    potential_paths.push(PathBuf::from(home).join(".npm/lib/node_modules/@anthropic-ai/claude-code/cli.js"));
                }
            },
            AiProvider::Copilot => {
                // Try environment variable (Windows)
                if let Ok(appdata) = std::env::var("APPDATA") {
                    potential_paths.push(PathBuf::from(appdata).join(r"npm\node_modules\@github\copilot\index.js"));
                }

                // Unix-like systems
                if let Ok(home) = std::env::var("HOME") {
                    potential_paths.push(PathBuf::from(home).join(".npm/lib/node_modules/@github/copilot/index.js"));
                }
            },
        }

        // 존재하는 첫 번째 경로 반환
        potential_paths.into_iter().find(|path| path.exists())
    }

    /// CLI 사용 가능 여부 확인
    pub fn verify_cli(provider: &AiProvider, path: Option<&str>) -> Result<String, String> {
        let cli_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            Self::detect_cli_path(provider)
                .ok_or_else(|| format!("{:?} CLI를 찾을 수 없습니다", provider))?
        };

        // 파일 존재 여부 확인
        if !cli_path.exists() {
            return Err(format!("CLI 파일을 찾을 수 없습니다: {:?}", cli_path));
        }

        // 간단한 테스트 명령 실행
        let test_result = match provider {
            AiProvider::Claude => {
                StdCommand::new("node")
                    .arg(&cli_path)
                    .arg("--version")
                    .output()
            },
            AiProvider::Copilot => {
                StdCommand::new("node")
                    .arg(&cli_path)
                    .arg("--version")
                    .output()
            },
        };

        match test_result {
            Ok(output) if output.status.success() => {
                Ok(format!("✓ {:?} CLI 사용 가능 (경로: {:?})", provider, cli_path))
            },
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("CLI 실행 실패: {}", stderr))
            },
            Err(e) => {
                Err(format!("Node.js 실행 실패: {}. Node.js가 설치되어 있는지 확인하세요.", e))
            },
        }
    }

    /// 설치 안내 메시지 가져오기
    pub fn get_installation_guide(provider: &AiProvider) -> String {
        match provider {
            AiProvider::Claude => {
                "Claude Code CLI 설치 방법:\n\n\
                1. 설치: npm install -g @anthropic-ai/claude-code\n\
                2. 로그인: claude login\n\n\
                자세한 내용: https://docs.anthropic.com/claude/docs/claude-code".to_string()
            },
            AiProvider::Copilot => {
                "GitHub Copilot CLI 설치 방법:\n\n\
                1. 설치: npm install -g @github/copilot\n\
                2. 로그인: gh copilot auth\n\n\
                참고: GitHub Copilot 구독이 필요합니다\n\
                자세한 내용: https://github.com/github/copilot-cli".to_string()
            },
        }
    }

    /// 프로바이더에게 질문하고 응답 받기
    pub fn ask(&self, question: &str) -> Result<String, String> {
        match self.provider {
            AiProvider::Claude => self.ask_claude(question),
            AiProvider::Copilot => self.ask_copilot(question),
        }
    }

    /// Claude Code CLI로 질문
    fn ask_claude(&self, question: &str) -> Result<String, String> {
        // CLI 경로: 설정값 또는 자동 탐지
        let claude_path = if let Some(ref path) = self.claude_path {
            PathBuf::from(path)
        } else {
            Self::detect_cli_path(&AiProvider::Claude)
                .ok_or_else(|| "Claude Code CLI를 찾을 수 없습니다. 설치 후 다시 시도하세요.".to_string())?
        };

        let output = StdCommand::new("node")
            .arg(claude_path)
            .arg("--print")
            .arg("--output-format")
            .arg("json")
            .arg(question)
            .output()
            .map_err(|e| format!("Failed to execute Claude: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Claude error: {}", error));
        }

        // Claude Code CLI는 JSON 형식으로 응답
        let response_str = String::from_utf8_lossy(&output.stdout).to_string();

        // JSON 파싱 시도
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_str) {
            if let Some(result) = json.get("result").and_then(|v| v.as_str()) {
                return Ok(result.to_string());
            } else if let Some(true) = json.get("is_error").and_then(|v| v.as_bool()) {
                return Err(json.get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string());
            }
        }

        // JSON 파싱 실패 시 원본 반환
        Ok(response_str.trim().to_string())
    }

    /// GitHub Copilot CLI로 질문
    fn ask_copilot(&self, question: &str) -> Result<String, String> {
        // CLI 경로: 설정값 또는 자동 탐지
        let copilot_path = if let Some(ref path) = self.copilot_path {
            PathBuf::from(path)
        } else {
            Self::detect_cli_path(&AiProvider::Copilot)
                .ok_or_else(|| "GitHub Copilot CLI를 찾을 수 없습니다. 설치 후 다시 시도하세요.".to_string())?
        };

        let output = StdCommand::new("node")
            .arg(copilot_path)
            .arg("-p")
            .arg(question)
            .arg("--allow-all-tools")
            .output()
            .map_err(|e| format!("Failed to execute Copilot: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Copilot error: {}", error));
        }

        let response = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(response.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_provider() {
        let config = AiConfig::default();
        assert_eq!(config.provider, AiProvider::Claude);
    }

    #[test]
    fn test_copilot_provider() {
        let config = AiConfig {
            provider: AiProvider::Copilot,
            claude_path: None,
            copilot_path: None, // Auto-detect from environment
        };

        // 실제 Copilot CLI가 설치되어 있어야 통과
        // let result = config.ask("What is 2+2?");
        // assert!(result.is_ok());
    }
}
