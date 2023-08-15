import { pool } from '@/lib/pg';

export default async function cc(req: any, res: any) {
    const {
        flows_user: flowsUser,
        flow_id: flowId,
    } = req.query;
  
    if (!flowsUser || !flowId) {
        return res.status(400).end('Bad request');
    }
  
    try {
        let keymap = await pool.query("SELECT * FROM webhook_keymap where flow_id = $1", [flowId]);
        let row = keymap.rows[0];

        if (row) {
            return res.json(row);
        }

        let lKey = makeKey(20);
        await pool.query(`
            INSERT INTO webhook_keymap (flows_user, flow_id, l_key)
            VALUES ($1, $2, $3)
          `, [flowsUser, flowId, lKey]);


        let r = {
          flow_id: flowId,
          flows_user: flowsUser,
          l_key: lKey
        };

        return res.json(r);
    } catch(e: any) {
        return res.status(500).end(e.toString());
    }
};

function makeKey(length: number) {
    var result           = '';
    var characters       = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    var charactersLength = characters.length;
    for ( var i = 0; i < length; i++ ) {
        result += characters.charAt(Math.floor(Math.random() * charactersLength));
    }
    return result;
}

