CREATE TABLE IF NOT EXISTS webhook_keymap (
    flow_id text PRIMARY KEY,
    flows_user text NOT NULL,
    l_key text NOT NULL,
    handler_fn text,
    UNIQUE(l_key)
);
