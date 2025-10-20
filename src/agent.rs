/*
 * The contents of this file are subject to the terms of the
 * Common Development and Distribution License, Version 1.0 only
 * (the "License").  You may not use this file except in compliance
 * with the License.
 *
 * See the file LICENSE in this distribution for details.
 * A copy of the CDDL is also available via the Internet at
 * http://www.opensource.org/licenses/cddl1.txt
 *
 * When distributing Covered Code, include this CDDL HEADER in each
 * file and include the contents of the LICENSE file from this
 * distribution.
 */

// Yet Another Youtube Down Loader
// - agent.rs file -

use ureq::{config::Config, Agent, Proxy};
use url::Url;

pub trait AgentBase {
    fn init(url: Url) -> Agent;
}

pub struct YaydlAgent;
impl AgentBase for YaydlAgent {
    // Default download agent for yaydl. Sets a proxy or not.
    fn init(url: Url) -> Agent {
        let mut agent_config = Config::builder().build();

        if let Some(env_proxy) = env_proxy::for_url(&url).host_port() {
            // Use a proxy:
            let proxy = Proxy::new(format!("{}:{}", env_proxy.0, env_proxy.1).as_ref()).unwrap();
            agent_config = Config::builder().proxy(Some(proxy)).build();
        }
        Agent::new_with_config(agent_config)
    }
}
