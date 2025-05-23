use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{
    services::{event_bus::EventBus, websocket::WebsocketService},
    User,
};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("Context to be set");

        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username),
            data_array: None,
        };

        if let Ok(_) = wss.tx.clone().try_send(serde_json::to_string(&message).unwrap()) {
            log::debug!("Registered successfully!");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.clone(),
                                avatar: format!("https://api.dicebear.com/8.x/adventurer-neutral/svg?seed={}", u),
                            })
                            .collect();
                        true
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData = serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        true
                    }
                    _ => false,
                }
            }
            Msg::SubmitMessage => {
                if let Some(input) = self.chat_input.cast::<HtmlInputElement>() {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self.wss.tx.clone().try_send(serde_json::to_string(&message).unwrap()) {
                        log::debug!("Send error: {:?}", e);
                    }
                    input.set_value("");
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let (cur_user, _) = ctx.link().context::<User>(Callback::noop()).expect("Context to be set");
        let cur_username = cur_user.username.borrow().clone();

        html! {
            <div class="flex w-screen h-screen overflow-hidden">
                <aside class="w-64 bg-gray-100 border-r overflow-y-auto">
                    <div class="text-xl font-semibold p-4 border-b">{"Users"}</div>
                    { for self.users.iter().map(|u| {
                        let is_self = u.name == cur_username;
                        let user_style = if is_self {
                            "bg-green-100 border-l-4 border-green-400"
                        } else {
                            "bg-white"
                        };
                        html! {
                            <div class={classes!("flex", "items-center", "m-3", "rounded-md", "p-2", "shadow-sm", user_style)}>
                                <img class="w-10 h-10 rounded-full mr-3" src={u.avatar.clone()} />
                                <div>
                                    <div class="font-medium text-sm">{&u.name}</div>
                                    <div class="text-xs text-gray-500">{"Hi there!"}</div>
                                </div>
                            </div>
                        }
                    })}
                </aside>

                <main class="flex-1 flex flex-col">
                    <header class="bg-white border-b p-4 text-xl font-bold">{"💬 Let's Chat!"}</header>

                    <section class="flex-1 overflow-y-auto px-4 py-6 space-y-4 bg-gray-50">
                        { for self.messages.iter().map(|m| {
                            let is_self = m.from == cur_username;
                            let avatar = self.users.iter().find(|u| u.name == m.from).map(|u| u.avatar.clone()).unwrap_or_default();
                            let alignment = if is_self { "justify-end" } else { "justify-start" };
                            let bubble_style = if is_self {
                                "bg-green-500 text-white rounded-l-2xl rounded-br-2xl"
                            } else {
                                "bg-gray-200 text-gray-900 rounded-r-2xl rounded-bl-2xl"
                            };

                            html! {
                                <div class={classes!("flex", alignment)}>
                                    <div class={classes!("flex", "items-start", "space-x-2", "max-w-md", "p-3", bubble_style)}>
                                        <img class="w-8 h-8 rounded-full" src={avatar} />
                                        <div>
                                            <div class="text-sm font-semibold">{&m.from}</div>
                                            <div class="mt-1 text-sm">
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html! { <img class="rounded-md max-w-[200px]" src={m.message.clone()} /> }
                                                    } else {
                                                        html! { <span>{&m.message}</span> }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        })}
                    </section>

                    <footer class="p-4 bg-white border-t flex items-center space-x-3 sticky bottom-0">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Type a message..."
                            class="flex-grow rounded-full border border-gray-300 p-2 px-4 focus:outline-none focus:ring-2 focus:ring-violet-400"
                        />
                        <button onclick={submit} class="bg-violet-600 hover:bg-violet-700 text-white rounded-full w-10 h-10 flex items-center justify-center transition">
                            <svg fill="currentColor" viewBox="0 0 24 24" class="w-5 h-5">
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </footer>
                </main>
            </div>
        }
    }
}
