import { pool } from '@/lib/pg';

export default async function cc(req: any, res: any) {
    const {
        l_key: lKey
    } = req.query;
 
    if (!lKey) {
        return res.status(400).end('Bad request');
    }
  
    try {
        let keymap = await pool.query("SELECT flows_user, flow_id, '__webhook__on_request_received' as handler_fn FROM webhook_keymap where l_key = $1", [lKey]);
        let row = keymap.rows[0];

        if (row) {
          return res.json(row);
        } else {
          return res.status(404).end('No flow binding with the key');
        }
    } catch(e: any) {
        return res.status(500).end(e.toString());
    }
};
