export const REDIRECT_URI='redirect_uri=https%3A%2F%2F05ce-34-84-78-213.jp.ngrok.io%2Fapi%2Fauth'

export async function getAuthedTeam(code: string) {
    let res = await fetch(`https://slack.com/api/oauth.v2.access?client_id=${process.env.SLACK_CLIENT_ID}&client_secret=${process.env.SLACK_CLIENT_SECRET}&code=${code}&${REDIRECT_URI}`);
    const access = await res.json();

    if (!access.ok) {
        throw 'Can not access user of slack';
    }

    /*
    res = await fetch('https://slack.com/api/users.profile.get', {
        headers: {
            Authorization: `Bearer ${access.authed_user.access_token}`
        }
    });
    const user = await res.json();
    */

    res = await fetch('https://slack.com/api/auth.test', {
        headers: {
            Authorization: `Bearer ${access.authed_user.access_token}`
        }
    });
    const auth = await res.json();

    return {
        user_id: access.authed_user.id,
        team: auth.url.match(/^https:\/\/(.*)\.slack\.com/)[1], 
        team_id: auth.team_id,
        access_token: access.access_token
    };
}

export async function getChannelByName(accessToken: string, teamId: string, channel: string) {
    let res = await fetch(`https://slack.com/api/conversations.list?types=public_channel,private_channel,im&team_id=${teamId}`, {
        headers: {
            Authorization: `Bearer ${accessToken}`
        }
    });
    const r = await res.json();

    if (!r.ok) {
        throw `Can not get channels of team ${teamId}`;
    }

    return r.channels.find((e:any) => e.name === channel);
}

export async function sendMessageToChannel(accessToken: string, channel: string, text: string) {
  await fetch('https://slack.com/api/chat.postMessage', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${accessToken}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      channel,
      text
    })
  });
}
