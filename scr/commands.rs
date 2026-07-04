//! Registro de comandos do Luna Code.
//!
//! Cada nome de comando é único. Tentar registrar um comando com um nome
//! já existente retorna LunaError::Internal.

use std::collections::HashMap;
use crate::{error::LunaError, types::{CommandId, CommandInfo}};

pub type CommandHandler = Box<dyn Fn(&[String]) -> Result<Option<String>, LunaError> + Send + Sync>;

struct CommandEntry {
    id: CommandId,
    name: String,
    description: String,
    handler: CommandHandler,
}

pub(crate) struct CommandRegistry {
    commands: HashMap<String, CommandEntry>,
    next_id: u64,
}

impl CommandRegistry {
    pub fn new() -> Self { Self { commands: HashMap::new(), next_id: 1 } }

    pub fn register<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) -> Result<CommandId, LunaError>
    where F: Fn(&[String]) -> Result<Option<String>, LunaError> + Send + Sync + 'static {
        let name = name.into();
        if self.commands.contains_key(&name) {
            return Err(LunaError::Internal(format!(
                "comando '{}' já está registrado; cancele o registro anterior antes de registrar novamente",
                name
            )));
        }
        let id = CommandId(self.next_id);
        self.next_id += 1;
        self.commands.insert(name.clone(), CommandEntry {
            id, name, description: description.into(), handler: Box::new(handler),
        });
        Ok(id)
    }

    pub fn unregister(&mut self, name: &str) { self.commands.remove(name); }

    pub fn execute(&self, name: &str, args: &[String]) -> Result<Option<String>, LunaError> {
        let entry = self.commands.get(name)
            .ok_or_else(|| LunaError::CommandNotFound(name.to_owned()))?;
        (entry.handler)(args).map_err(|e| LunaError::CommandExecutionFailed {
            name: name.to_owned(), reason: e.to_string(),
        })
    }

    pub fn list(&self) -> Vec<CommandInfo> {
        let mut infos: Vec<CommandInfo> = self.commands.values().map(|e| CommandInfo {
            id: e.id, name: e.name.clone(), description: e.description.clone(),
        }).collect();
        infos.sort_by(|a, b| a.name.cmp(&b.name));
        infos
    }

    #[allow(dead_code)]
    pub fn contains(&self, name: &str) -> bool { self.commands.contains_key(name) }
}
