class SignalController:
    signals = {
        "project-open": [],
        "project-update": [],
        "todo-new": [],
        "todo-checked": [],
        "todo-duration-update": [],
        "todo-toggle_completed": [],
    }
    blocked_signals = []

    def add_handler(self, signal: str, handler):
        self.signals[signal].append(handler)

    def emit_signal(self, signal: str, *params):
        for handler in self.signals[signal]:
            handler(*params)

    def block_signal(self, signal: str):
        if signal not in self.blocked_signals:
            self.blocked_signals.append(signal)

    def unblock_signal(self, signal: str):
        if signal in self.blocked_signals:
            self.blocked_signals.remove(signal)

