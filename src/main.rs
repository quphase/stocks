use std::collections::HashMap;
use yew::prelude::*;

pub mod csv_parser;
mod tax;

use web_sys::{Event, HtmlInputElement};
use yew::{html, html::TargetCast, Component, Context, Html};

use gloo_file::callbacks::FileReader;
use gloo_file::File;

enum Msg {
    Loaded(String, String),
    File(File),
    Err(String),
    UpdateSymbolFilter(String),
}

struct Model {
    tax_info: Option<tax::AllInfo>,
    symbol_filter: String,
    reader: Option<FileReader>,
    err: String,
    csv_data: Option<csv_parser::Trades>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            tax_info: None,
            symbol_filter: String::new(),
            err: String::new(),
            reader: None,
            csv_data: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(_fname, data) => {
                match csv_parser::parse(&data) {
                    Ok(trades) => {
                        self.csv_data = Some(trades.clone());
                        let tax_info = tax::parse(&trades, self.symbol_filter.clone());
                        self.tax_info = Some(tax_info);
                    }
                    Err(csv_err) => {
                        self.err = format!("{:?}", csv_err);
                    }
                };
                self.reader = None;
                true
            }
            Msg::File(file) => {
                let file_name = file.name();
                let task = {
                    let file_name = file_name.clone();
                    let link = ctx.link().clone();
                    gloo_file::callbacks::read_as_text(&file, move |res| {
                        link.send_message(Msg::Loaded(
                            file_name,
                            res.unwrap_or_else(|e| e.to_string()),
                        ))
                    })
                };
                self.reader = Some(task);
                true
            }
            Msg::UpdateSymbolFilter(c) => {
                self.symbol_filter = c.to_uppercase();
                if let Some(trades) = &self.csv_data {
                    let tax_info = tax::parse(&trades, self.symbol_filter.clone());
                    self.tax_info = Some(tax_info);
                }
                true
            }
            Msg::Err(err) => {
                self.err = err;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        //let link = ctx.link();

        let mut information = html! {};
        if let Some(info) = &self.tax_info {
            let earnings: f64 = info
                .iter()
                .map(|(s, data)| {
                    let mut sum = 0.;
                    for d in data {
                        match d {
                            tax::Information::PriceDiff(a) => sum += a,
                            _ => (),
                        };
                    }
                    sum
                })
                .sum();
            information =
                html! { format!("Total capital earnings: {}", (earnings * 100.).round()/100.) };
        }

        html! {
                    <div>
                        <div class="flex mt-8">
            <div class="max-w-2xl rounded-lg  bg-white">
                <div class="m-4">
                    <label class="inline-block mb-2 text-gray-500">{"File Upload"}</label>
                    <div class="flex items-center justify-center w-full">
                        <label
                            class="flex flex-col w-full h-32 border-4 border-blue-200 border-dashed hover:bg-gray-100 hover:border-gray-300">
                            <div class="flex flex-col items-center justify-center pt-7">
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-gray-400 group-hover:text-gray-600"
                                    fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                                </svg>
                                <p class="pt-1 text-sm tracking-wider text-gray-400 group-hover:text-gray-600">
                                {"Attach a file"}</p>
                            </div>
                            <input type="file" class="opacity-0" onchange={ctx.link().callback(move |e: Event| {
                                    let input: HtmlInputElement = e.target_unchecked_into();

                                    if let Some(files) = input.files() {
                                        let file = files.get(0).unwrap();
                                        let result = File::from(web_sys::File::from(file));
                                        Msg::File(result)
                                    }
                                    else {
                                        Msg::Err("Something went wrong with upload".to_string())
                                    }
                                })}/>
                        </label>
                    </div>
                </div>
            </div>
        </div>
                        <div class="m-4">
                            <p class="mb-2 text-gray-500">{ "Filter by stock ticker" }</p>
                            <input class="bg-gray-40 border-2 border-blue-200 p-2" placeholder="GME" type="text" value={self.symbol_filter.clone()} oninput={ctx.link().callback(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::UpdateSymbolFilter(input.value())
                            }
                            )}/>
                        </div>
                        { information }
                        if let Some(info) = &self.tax_info {
                            <div>{ for info.iter().map(|f| Self::view_tax(f)) }</div>
                        }
                        { &self.err }
                    </div>
                }
    }
}

impl Model {
    fn view_tax(data: (&String, &Vec<tax::Information>)) -> Html {
        let (symbol, information) = data;

        let mut sum = 0.;
        for info in information {
            match info {
                tax::Information::PriceDiff(a) => sum += a,
                _ => (),
            }
        }

        let color_class = if sum > 0. {
            "border-green-700"
        } else if sum == 0.0 {
            "border-gray-700"
        } else {
            "border-red-700"
        };

        html! {
            <div>
                //<div> { format!("{:?}", buys_and_sells) } </div>
                <div class={classes!("m-4", "w-96", "bg-gray-200","border-r-8", color_class)}>
                    <h2 class="text-2xl font-medium leading-tight"> {symbol}</h2>
                    {for information.iter().map(|f| Self::view_information(f))}
                </div>
            </div>
        }
    }

    fn view_information(data: &tax::Information) -> Html {
        html! {
            <div class="w-96">
            {
            match data {
                tax::Information::Buy(q, p, d) =>
                    html! {
                        <div class="bg-blue-400 rounded-md p-1 m-2"> { format!("Buy: {}", q) } </div>
                    },
                tax::Information::Sell(q, p, d) =>
                    html! {
                        <div class="bg-indigo-400 rounded p-1 mt-2 mr-2 ml-2"> { format!("Sell: {}", q) } </div>
                    },
                tax::Information::TimeDiff(d) =>
                    html! {
                        <div class="w-80 bg-indigo-200 p-1 ml-8"> { format!("{} days -- {}", d.num_days(), if d.num_days() < 365 { "short-term capital" } else { "long-term capital" }) } </div>
                    },

                 tax::Information::PriceDiff(a) =>
                    html! {
                        if a > &0. {
                            <div class="w-64 bg-green-200 p-1 ml-24"> { format!("${}", (a*100.).round()/100.)} </div>
                        }
                        else {
                            <div class="w-64 bg-red-200 p-1 ml-24"> { format!("${}", (a*100.).round()/100.)} </div>
                        }
                    },
                tax::Information::Remaing(q) =>
                    html! {
                        <div>
                            <hr/>
                            <div class="w-96 bg-black text-white"> { format!("Quantity Owned: {}", q) } </div>
                        </div>
                    },


                _ => html! {}
            }
            }
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}