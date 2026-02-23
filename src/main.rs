use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[component]
fn App() -> Element {
    let mut display = use_signal(|| "0".to_string());
    let mut stored: Signal<f64> = use_signal(|| 0.0);
    let mut pending_op: Signal<Option<Op>> = use_signal(|| None);
    let mut reset_on_next: Signal<bool> = use_signal(|| false);

    let mut handle_digit = move |d: &'static str| {
        if *reset_on_next.read() {
            display.set(d.to_string());
            reset_on_next.set(false);
        } else {
            let cur = display.read().clone();
            if cur == "0" {
                display.set(d.to_string());
            } else {
                display.set(format!("{}{}", cur, d));
            }
        }
    };

    let mut handle_decimal = move |_| {
        if *reset_on_next.read() {
            display.set("0.".to_string());
            reset_on_next.set(false);
        } else if !display.read().contains('.') {
            let cur = display.read().clone();
            display.set(format!("{}.", cur));
        }
    };

    let mut handle_op = move |op: Op| {
        let val: f64 = display.read().parse().unwrap_or(0.0);
        let current_op = pending_op.read().clone(); // borrow ends here
        if let Some(ref pending) = current_op {
            let result = match pending {
                Op::Add => *stored.read() + val,
                Op::Sub => *stored.read() - val,
                Op::Mul => *stored.read() * val,
                Op::Div => {
                    if val == 0.0 { f64::NAN } else { *stored.read() / val }
                }
            };
            stored.set(result);
            let formatted = format_num(result);
            display.set(formatted);
        } else {
            stored.set(val);
        }
        pending_op.set(Some(op));
        reset_on_next.set(true);
    };

    let mut handle_equals = move |_| {
        let val: f64 = display.read().parse().unwrap_or(0.0);
        let current_op = pending_op.read().clone(); // borrow ends here
        if let Some(ref op) = current_op {
            let result = match op {
                Op::Add => *stored.read() + val,
                Op::Sub => *stored.read() - val,
                Op::Mul => *stored.read() * val,
                Op::Div => {
                    if val == 0.0 { f64::NAN } else { *stored.read() / val }
                }
            };
            display.set(format_num(result));
            stored.set(0.0);
            pending_op.set(None);
            reset_on_next.set(true);
        }
    };

    let mut handle_clear = move |_| {
        display.set("0".to_string());
        stored.set(0.0);
        pending_op.set(None);
        reset_on_next.set(false);
    };

    let mut handle_toggle_sign = move |_| {
        let val: f64 = display.read().parse().unwrap_or(0.0);
        display.set(format_num(-val));
    };

    let mut handle_percent = move |_| {
        let val: f64 = display.read().parse().unwrap_or(0.0);
        display.set(format_num(val / 100.0));
    };

    let has_op = pending_op.read().is_some();

    rsx! {
        style { {STYLES} }
        div { class: "calculator",
            // Display
            div { class: "display",
                span { class: "display-text", "{display}" }
            }
            // Row 1
            div { class: "row",
                button { class: "btn fn-btn", onclick: handle_clear, "AC" }
                button { class: "btn fn-btn", onclick: handle_toggle_sign, "+/-" }
                button { class: "btn fn-btn", onclick: handle_percent, "%" }
                button { class: if has_op { "btn op-btn active" } else { "btn op-btn" },
                    onclick: move |_| handle_op(Op::Div), "÷"
                }
            }
            // Row 2
            div { class: "row",
                button { class: "btn num-btn", onclick: move |_| handle_digit("7"), "7" }
                button { class: "btn num-btn", onclick: move |_| handle_digit("8"), "8" }
                button { class: "btn num-btn", onclick: move |_| handle_digit("9"), "9" }
                button { class: "btn op-btn", onclick: move |_| handle_op(Op::Mul), "×" }
            }
            // Row 3
            div { class: "row",
                button { class: "btn num-btn", onclick: move |_| handle_digit("4"), "4" }
                button { class: "btn num-btn", onclick: move |_| handle_digit("5"), "5" }
                button { class: "btn num-btn", onclick: move |_| handle_digit("6"), "6" }
                button { class: "btn op-btn", onclick: move |_| handle_op(Op::Sub), "−" }
            }
            // Row 4
            div { class: "row",
                button { class: "btn num-btn", onclick: move |_| handle_digit("1"), "1" }
                button { class: "btn num-btn", onclick: move |_| handle_digit("2"), "2" }
                button { class: "btn num-btn", onclick: move |_| handle_digit("3"), "3" }
                button { class: "btn op-btn", onclick: move |_| handle_op(Op::Add), "+" }
            }
            // Row 5
            div { class: "row",
                button { class: "btn num-btn wide", onclick: move |_| handle_digit("0"), "0" }
                button { class: "btn num-btn", onclick: handle_decimal, "." }
                button { class: "btn eq-btn", onclick: handle_equals, "=" }
            }
        }
    }
}

fn format_num(val: f64) -> String {
    if val.is_nan() {
        return "Error".to_string();
    }
    if val.is_infinite() {
        return "∞".to_string();
    }
    // Remove trailing zeros after decimal
    if val.fract() == 0.0 && val.abs() < 1e12 {
        format!("{}", val as i64)
    } else {
        // Limit decimal places
        let s = format!("{:.10}", val);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

const STYLES: &str = r#"
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

body {
    background: #1a1a2e;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
}

.calculator {
    background: #16213e;
    border-radius: 24px;
    padding: 20px;
    width: 320px;
    box-shadow: 0 25px 60px rgba(0,0,0,0.5), 0 0 0 1px rgba(255,255,255,0.05);
}

.display {
    background: #0f3460;
    border-radius: 16px;
    padding: 20px 24px;
    margin-bottom: 16px;
    min-height: 80px;
    display: flex;
    align-items: flex-end;
    justify-content: flex-end;
    overflow: hidden;
}

.display-text {
    color: #e2e8f0;
    font-size: 2.8rem;
    font-weight: 300;
    letter-spacing: -1px;
    word-break: break-all;
    text-align: right;
    line-height: 1;
}

.row {
    display: flex;
    gap: 10px;
    margin-bottom: 10px;
}

.btn {
    flex: 1;
    height: 68px;
    border: none;
    border-radius: 16px;
    font-size: 1.4rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.1s ease;
    outline: none;
    position: relative;
    overflow: hidden;
}

.btn:active {
    transform: scale(0.94);
}

.btn::after {
    content: '';
    position: absolute;
    inset: 0;
    background: rgba(255,255,255,0.1);
    opacity: 0;
    border-radius: inherit;
    transition: opacity 0.1s;
}

.btn:hover::after {
    opacity: 1;
}

.num-btn {
    background: #1a1a2e;
    color: #e2e8f0;
    border: 1px solid rgba(255,255,255,0.08);
}

.fn-btn {
    background: #533483;
    color: #e2e8f0;
}

.op-btn {
    background: #e94560;
    color: white;
}

.op-btn.active {
    background: white;
    color: #e94560;
}

.eq-btn {
    background: linear-gradient(135deg, #e94560, #c62a47);
    color: white;
    flex: 1;
}

.wide {
    flex: 2.2;
    justify-content: flex-start;
    padding-left: 26px;
    display: flex;
    align-items: center;
}
"#;
