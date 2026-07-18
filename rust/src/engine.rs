use rquickjs::{Context, Error as JsError, Function, Object, Runtime, Value};

use crate::event::EventRecord;

/// Persistent JS realm that statements are evaluated against.
///
/// Top-level `let`/`const`/`var`/`function`/`class` declarations survive across
/// separate `eval` calls as long as they share this `Context` — that's the
/// mechanism the original `state.js` achieved by hand-generating `state.<name>`
/// member expressions and `eval`-ing them. Embedding a real JS engine gives us
/// that persistence, plus full JS control flow (if/for/function/class), for free.
pub struct Engine {
    context: Context,
    _runtime: Runtime,
}

impl Engine {
    pub fn new() -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;
        let context = Context::full(&runtime).map_err(|e| e.to_string())?;

        // `event(name, data)` is exposed to user statements the same way the
        // original does it: as a name that's simply in scope when statements
        // are evaluated, backed by a buffer we drain after each statement.
        context
            .with(|ctx| {
                ctx.eval::<(), _>(
                    "globalThis.__nuc_events = []; \
                     function event(name, data) { \
                       __nuc_events.push({ name: name, data: JSON.stringify(data) }); \
                     }",
                )
            })
            .map_err(|e| e.to_string())?;

        Ok(Engine {
            context,
            _runtime: runtime,
        })
    }

    /// Evaluate one statement. Returns the JSON text of its completion value,
    /// or `None` when the completion value is `undefined`.
    pub fn eval(&self, source: &str) -> Result<Option<String>, String> {
        self.context.with(|ctx| {
            let raw: Value = ctx.eval(source).map_err(|err| describe_exception(&ctx, err))?;

            if raw.is_undefined() {
                return Ok(None);
            }

            let json: Object = ctx
                .globals()
                .get("JSON")
                .map_err(|err| describe_exception(&ctx, err))?;
            let stringify: Function = json
                .get("stringify")
                .map_err(|err| describe_exception(&ctx, err))?;
            let text: String = stringify
                .call((raw,))
                .map_err(|err| describe_exception(&ctx, err))?;

            Ok(Some(text))
        })
    }

    /// Current value of a bare identifier, as JSON, or `None` if it is
    /// undefined (or was never declared).
    pub fn read(&self, name: &str) -> Result<Option<String>, String> {
        self.eval(&format!(
            "(typeof {name} === 'undefined' ? undefined : {name})"
        ))
    }

    /// Assign a JSON-encoded value back onto a bare identifier, used to
    /// restore a snapshot on rollback. `var` re-declaration is used so this
    /// is safe even if the target was never declared before.
    pub fn restore(&self, name: &str, json: &Option<String>) -> Result<(), String> {
        match json {
            Some(value) => self.eval(&format!("{name} = ({value});")).map(|_| ()),
            None => self.eval(&format!("var {name} = undefined;")).map(|_| ()),
        }
    }

    /// Drain the events recorded via `event(name, data)` since the last drain.
    pub fn drain_events(&self) -> Result<Vec<EventRecord>, String> {
        let json = self.eval(
            "(function() { \
               var events = __nuc_events; \
               __nuc_events = []; \
               return events; \
             })()",
        )?;

        match json {
            Some(text) => serde_json::from_str(&text).map_err(|e| e.to_string()),
            None => Ok(Vec::new()),
        }
    }
}

/// `catch()`-based exception introspection. The exact shape of a thrown value
/// is uncertain (Error object vs primitive), so this degrades gracefully
/// through a few fallbacks rather than assuming a `.message` property exists.
fn describe_exception(ctx: &rquickjs::Ctx<'_>, err: JsError) -> String {
    if !matches!(err, JsError::Exception) {
        return err.to_string();
    }

    let exc = ctx.catch();

    if let Some(exception) = exc.as_exception() {
        if let Some(message) = exception.message() {
            return message;
        }
        return exception.to_string();
    }

    if let Some(obj) = exc.as_object() {
        if let Ok(message) = obj.get::<_, String>("message") {
            return message;
        }
    }

    if let Some(s) = exc.as_string() {
        if let Ok(text) = s.to_string() {
            return text;
        }
    }

    "unknown JS exception".to_string()
}
