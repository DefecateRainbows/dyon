fn new_window() -> {
    return {title: "(no title)"}
}

fn title(window: 'return) -> {
    return window.title
}

fn title(mut window, val: 'window) {
    window.title = val
}

fn main() {
    window := new_window()
    println(title(window))
    title(mut window, "hello world!")
    println(title(window))
}
