use crate::outbox::Outbox;
use crate::protocol::{AskParams, Started, SteerParams, TurnReference};
use crate::turns::Turns;
use anyhow::Result;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{Json, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use std::path::PathBuf;

const INSTRUCTIONS: &str = "Use ask for implementation and follow-up work in the persistent Codex App task. Use the codex CLI directly for parallel or disposable research; the insd47:codex skill guides that workflow. Pass the threadId and turnId returned by ask to status, steer, or interrupt. ask returns after dispatch; completion arrives through the Codex App monitor. Calling ask again steers the active persistent turn automatically.";

#[derive(Clone)]
pub struct Server {
    turns: Turns,
    tool_router: ToolRouter<Self>,
}

#[tool_router(router = tool_router)]
impl Server {
    pub fn new(cwd: PathBuf, outbox: Outbox) -> Self {
        Self {
            turns: Turns::new(cwd, outbox),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "ask",
        description = "Send a prompt to the visible persistent Codex App task. A repeated call steers its active turn.",
        annotations(
            title = "Ask Codex App",
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn ask(
        &self,
        Parameters(params): Parameters<AskParams>,
    ) -> Result<Json<Started>, String> {
        self.turns
            .ask(params.prompt)
            .await
            .map(Json)
            .map_err(|error| format!("{error:#}"))
    }

    #[tool(
        name = "status",
        description = "Check whether a Codex turn is still owned by the active persistent plugin task.",
        annotations(
            title = "Check Codex Turn",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn status(
        &self,
        Parameters(reference): Parameters<TurnReference>,
    ) -> Result<Json<TurnReference>, String> {
        self.turns
            .status(reference)
            .await
            .map(Json)
            .map_err(|error| format!("{error:#}"))
    }

    #[tool(
        name = "steer",
        description = "Redirect one active Codex turn identified by its native thread and turn IDs.",
        annotations(
            title = "Steer Codex Turn",
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn steer(
        &self,
        Parameters(params): Parameters<SteerParams>,
    ) -> Result<Json<TurnReference>, String> {
        self.turns
            .steer(params)
            .await
            .map(Json)
            .map_err(|error| format!("{error:#}"))
    }

    #[tool(
        name = "interrupt",
        description = "Interrupt one active Codex turn identified by its native thread and turn IDs.",
        annotations(
            title = "Interrupt Codex Turn",
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn interrupt(
        &self,
        Parameters(reference): Parameters<TurnReference>,
    ) -> Result<Json<TurnReference>, String> {
        self.turns
            .interrupt(reference)
            .await
            .map(Json)
            .map_err(|error| format!("{error:#}"))
    }
}

impl Server {
    pub async fn run(self) -> Result<()> {
        self.serve((tokio::io::stdin(), tokio::io::stdout()))
            .await?
            .waiting()
            .await?;
        Ok(())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("codex-app", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_exposes_only_the_four_persistent_tools() {
        let root = std::env::temp_dir().join(format!("codex-app-server-{}", uuid::Uuid::new_v4()));
        let server = Server::new(root.clone(), Outbox::new(&root, &root));
        let mut names = server
            .tool_router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.into_owned())
            .collect::<Vec<_>>();
        names.sort_unstable();
        assert_eq!(names, ["ask", "interrupt", "status", "steer"]);
        assert_eq!(
            server.get_info().instructions.as_deref(),
            Some(INSTRUCTIONS)
        );
    }
}
