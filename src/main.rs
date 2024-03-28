const MAX_DURATION: std::time::Duration = std::time::Duration::from_millis(50);
const SCRIPT: &str = r#"
    export default function handler() {
        let sum = 0;
        for (let i = 0; i < 150_000; i++) {
            sum += i;
        }
        return JSON.stringify({ sum });
    }
"#;

struct Main {
    runtime: rquickjs::Runtime,
    context: rquickjs::Context,
}

impl Main {
    fn new() -> Self {
        let runtime = rquickjs::Runtime::new().unwrap();
        runtime.set_memory_limit(128 * 1_024 * 1_024);
        runtime.set_max_stack_size(512 * 1_024);
        let context = rquickjs::Context::full(&runtime).unwrap();
        Self { runtime, context }
    }

    fn start_interrupt_handler(&self) {
        let start = std::time::Instant::now();
        self.runtime.set_interrupt_handler(Some(Box::new(move || {
            let elapsed = start.elapsed();
            println!("[*] Interrupt Check - Elapsed: {:?}", elapsed);
            elapsed > MAX_DURATION
        })));
    }

    fn clear_interrupt_handler(&self) {
        self.runtime.set_interrupt_handler(None);
    }

    pub fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce(rquickjs::Ctx) -> R,
    {
        self.start_interrupt_handler();
        let result = self.context.with(f);
        self.clear_interrupt_handler();
        result
    }
}

fn main() -> anyhow::Result<()> {
    let main = Main::new();

    println!("[*] Compiling function");
    let func = main.run(|ctx| -> rquickjs::Persistent<rquickjs::Function> {
        let module = ctx.clone().compile("script", SCRIPT).unwrap();
        let func: rquickjs::Function = module.get("default").unwrap();
        rquickjs::Persistent::save(&ctx, func)
    });

    println!("[*] Running function");
    main.run(|ctx| {
        let func = func.restore(&ctx).unwrap();
        let result = func.call::<_, String>(());
        match result {
            Ok(result) => {
                println!("[*] Result: {}", result);
            }
            Err(err) => {
                println!("[*] Error while calling function: {}", err);
            }
        }
    });

    Ok(())
}
