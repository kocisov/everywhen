use std::io::Read;

#[derive(serde::Serialize, serde::Deserialize)]
struct EventData {
    t: String,
    d: serde_json::Value,
}

struct VM {
    runtime: rquickjs::Runtime,
    context: rquickjs::Context,
}

impl VM {
    fn new() -> Self {
        let runtime = rquickjs::Runtime::new().unwrap();
        runtime.set_memory_limit(128 * 1_024 * 1_024);
        runtime.set_max_stack_size(512 * 1_024);
        let context = rquickjs::Context::full(&runtime).unwrap();
        Self { runtime, context }
    }

    fn start_interrupt_handler(&self, max_duration: std::time::Duration) {
        let start = std::time::Instant::now();
        self.runtime.set_interrupt_handler(Some(Box::new(move || {
            let elapsed = start.elapsed();
            let cond = elapsed > max_duration;
            println!("[*] Elapsed: {:?}, Max: {:?}", elapsed, max_duration);
            if cond {
                println!("[*] Interrupted after {:?}", elapsed);
            }
            cond
        })));
    }

    fn clear_interrupt_handler(&self) {
        self.runtime.set_interrupt_handler(None);
    }

    pub fn run<F, R>(&self, f: F, max_duration: std::time::Duration) -> R
    where
        F: FnOnce(rquickjs::Ctx) -> R,
    {
        self.start_interrupt_handler(max_duration);
        let result = self.context.with(f);
        self.clear_interrupt_handler();
        result
    }
}

fn main() -> anyhow::Result<()> {
    let vm = VM::new();
    let mut source = String::new();
    let max_duration = std::time::Duration::from_millis(50);
    std::io::stdin().read_to_string(&mut source)?;

    println!("[*] Compiling function");
    let func = vm.run(
        |ctx| {
            let module = ctx.clone().compile("source", source).unwrap();
            let func: rquickjs::Function = module.get("default").unwrap();
            rquickjs::Persistent::save(&ctx, func)
        },
        max_duration,
    );

    let now = std::time::SystemTime::now();
    let timestamp = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let data = &EventData {
        t: "test".to_string(),
        d: serde_json::json!({
         "timestamp": timestamp,
        }),
    };

    println!("[*] Running function");
    let result = vm.run(
        |ctx| {
            let func = func.restore(&ctx).unwrap();
            func.call::<_, String>((serde_json::to_string(data).unwrap(),))
                .unwrap()
        },
        max_duration,
    );

    println!("[*] Result: {}", result);
    Ok(())
}
