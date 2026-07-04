use std::collections::HashMap;

use crate::{
    error::LunaError,
    types::{CommandId, CommandInfo},
};

pub(crate) type CommandHandler =
    Box<dyn Fn(&[String]) -> Result<Option<String>, LunaError> + Send + Sync>;

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
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn register<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) -> Result<CommandId, LunaError>
    where
        F: Fn(&[String]) -> Result<Option<String>, LunaError> + Send + Sync + 'static,
    {
        let name = name.into();
        if self.commands.contains_key(&name) {
            return Err(LunaError::CommandAlreadyRegistered(name));
        }

        let id = CommandId(self.next_id);
        self.next_id += 1;
        self.commands.insert(
            name.clone(),
            CommandEntry {
                id,
                name,
                description: description.into(),
                handler: Box::new(handler),
            },
        );
        Ok(id)
    }

    pub fn unregister(&mut self, name: &str) {
        self.commands.remove(name);
    }

    pub fn execute(&self, name: &str, args: &[String]) -> Result<Option<String>, LunaError> {
        let entry = self
            .commands
            .get(name)
            .ok_or_else(|| LunaError::CommandNotFound(name.to_owned()))?;

        (entry.handler)(args).map_err(|e| LunaError::CommandExecutionFailed {
            name: name.to_owned(),
            reason: e.to_string(),
        })
    }

    pub fn list(&self) -> Vec<CommandInfo> {
        let mut infos: Vec<CommandInfo> = self
            .commands
            .values()
            .map(|e| CommandInfo {
                id: e.id,
                name: e.name.clone(),
                description: e.description.clone(),
            })
            .collect();
        infos.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        infos
    }

    pub fn contains(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_execute_command() {
        let mut registry = CommandRegistry::new();
        registry.register("test.hello", "Diz olá", |_args| {
            Ok(Some("olá, mundo!".to_owned()))
        }).unwrap();

        let result = registry.execute("test.hello", &[]).unwrap();
        assert_eq!(result, Some("olá, mundo!".to_owned()));
    }

    #[test]
    fn duplicate_registration_returns_correct_error() {
        let mut registry = CommandRegistry::new();
        registry.register("cmd.a", "Primeiro", |_| Ok(None)).unwrap();
        let err = registry.register("cmd.a", "Duplicado", |_| Ok(None)).unwrap_err();
        assert!(
            matches!(err, LunaError::CommandAlreadyRegistered(_)),
            "esperava CommandAlreadyRegistered, recebeu: {err:?}"
        );
    }

    #[test]
    fn unregister_then_reregister_succeeds() {
        let mut registry = CommandRegistry::new();
        registry.register("cmd.b", "Original", |_| Ok(None)).unwrap();
        registry.unregister("cmd.b");
        let result = registry.register("cmd.b", "Novo", |_| Ok(None));
        assert!(result.is_ok());
    }

    #[test]
    fn execute_unknown_command_returns_error() {
        let registry = CommandRegistry::new();
        assert!(matches!(
            registry.execute("nao.existe", &[]),
            Err(LunaError::CommandNotFound(_))
        ));
    }

    #[test]
    fn command_receives_args() {
        let mut registry = CommandRegistry::new();
        registry.register("echo", "Repete o argumento", |args| {
            Ok(Some(args.join(" ")))
        }).unwrap();

        let result = registry
            .execute("echo", &["luna".to_owned(), "code".to_owned()])
            .unwrap();
        assert_eq!(result, Some("luna code".to_owned()));
    }

    #[test]
    fn list_commands_sorted() {
        let mut registry = CommandRegistry::new();
        registry.register("z.cmd", "Z", |_| Ok(None)).unwrap();
        registry.register("a.cmd", "A", |_| Ok(None)).unwrap();
        registry.register("m.cmd", "M", |_| Ok(None)).unwrap();

        let names: Vec<_> = registry.list().iter().map(|c| c.name.clone()).collect();
        assert_eq!(names, vec!["a.cmd", "m.cmd", "z.cmd"]);
    }

    #[test]
    fn contains_reflects_registration_state() {
        let mut registry = CommandRegistry::new();
        assert!(!registry.contains("x.cmd"));
        registry.register("x.cmd", "X", |_| Ok(None)).unwrap();
        assert!(registry.contains("x.cmd"));
        registry.unregister("x.cmd");
        assert!(!registry.contains("x.cmd"));
    }
}
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
