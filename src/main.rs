use yew::prelude::*;

pub mod csv_parser;
mod tax;

use web_sys::{Event, HtmlInputElement};
use yew::{html, html::TargetCast, Component, Context, Html};

use gloo_file::callbacks::FileReader;
use gloo_file::File;

use chrono::TimeZone;

enum Msg {
    Loaded(String, String),
    File(File),
    Err(String),
    UpdateSymbolFilter(String),
    UpdateYearFilter(String),
}

struct Model {
    stock_tax_info: Option<tax::AllInfo>,
    stock_csv_data: Option<csv_parser::Trades>,

    reader: Option<FileReader>,
    symbol_filter: String,
    err: String,
    year: Option<chrono::DateTime<chrono::Utc>>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            stock_tax_info: None,
            stock_csv_data: None,

            reader: None,
            symbol_filter: String::new(),
            err: String::new(),
            year: Some(chrono::Utc.ymd(2021, 1, 1).and_hms(0, 0, 0)), //year: None
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(_fname, data) => {
                match csv_parser::parse(&data) {
                    Ok(trades) => {
                        self.stock_csv_data = Some(trades.clone());
                        let stock_tax_info =
                            tax::parse(&trades, self.symbol_filter.clone(), self.year);
                        self.stock_tax_info = Some(stock_tax_info);
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
                if let Some(trades) = &self.stock_csv_data {
                    let stock_tax_info = tax::parse(&trades, self.symbol_filter.clone(), self.year);
                    self.stock_tax_info = Some(stock_tax_info);
                }
                true
            }
            Msg::UpdateYearFilter(y) => {
                if y == "none" {
                    self.year = None;
                } else {
                    if let Some(year) = y.parse().ok() {
                        self.year = Some(chrono::Utc.ymd(year, 1, 1).and_hms(0, 0, 0));
                    }
                }
                if let Some(trades) = &self.stock_csv_data {
                    let stock_tax_info = tax::parse(&trades, self.symbol_filter.clone(), self.year);
                    self.stock_tax_info = Some(stock_tax_info);
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
        if let Some(info) = &self.stock_tax_info {
            let earnings: f64 = info
                .iter()
                .map(|(_s, data)| {
                    let mut sum = 0.;
                    for d in data {
                        match d {
                            tax::Information::PriceDiff(a, _d) => sum += a,
                            tax::Information::Fees(f) => sum -= f,
                            _ => (),
                        };
                    }
                    sum
                })
                .sum();
            information = html! {
                <div class="dark:text-white">
                    { format!("Total capital earnings: ${}", (earnings * 100.).round()/100.) }
                </div>
            };
        }

        html! {
            <>
                <main class="flex-grow">
        <div>
            <h1 class="text-3xl font-medium leading-tight mt-0 mb-2 text-blue-600 dark:text-white">{"client-sided stock tax analyzer"}</h1>
            <div class="flex mt-8">
                <div class="max-w-2xl rounded-lg  bg-white dark:bg-gray-900">
                    <div class="m-4">
                        <label class="inline-block mb-2 text-gray-500 dark:text-gray-100">{"Upload Stock History"}</label>
                            <div class="flex items-center justify-center w-full">
                                <label
                                    class="flex flex-col w-full h-32 border-4 border-blue-200 dark:border-blue-800 border-dashed hover:bg-gray-100 hover:border-gray-300 dark:hover:bg-gray-800">
                                    <div class="flex flex-col items-center justify-center pt-7">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-gray-400 dark:text-gray-100 group-hover:text-gray-600"
                                            fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                                        </svg>
                                        <p class="pt-1 text-sm tracking-wider text-gray-400 dark:text-gray-100 group-hover:text-gray-600">
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
                <div class="max-w-2xl rounded-lg  bg-white dark:bg-gray-900">
                    <div class="m-4">
                        <label class="inline-block mb-2 text-gray-500 dark:text-gray-100">{"Upload Crypto History"}</label>
                            <div class="flex items-center justify-center w-full">
                                <label
                                    class="flex flex-col w-full h-32 border-4 border-blue-200 dark:border-blue-800 border-dashed hover:bg-gray-100 hover:border-gray-300 dark:hover:bg-gray-800">
                                    <div class="flex flex-col items-center justify-center pt-7">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-gray-400 dark:text-gray-100 group-hover:text-gray-600"
                                            fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                                        </svg>
                                        <p class="pt-1 text-sm tracking-wider text-gray-400 dark:text-gray-100 group-hover:text-gray-600">
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
                    <p class="mb-2 text-gray-500 dark:text-gray-100">{ "Filter by stock ticker" }</p>
                    <input class="bg-gray-40 dark:bg-gray-800 dark:text-gray-100 border-2 border-blue-200 dark:border-blue-800 p-2" placeholder="GME" type="text" value={self.symbol_filter.clone()} oninput={ctx.link().callback(move |e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateSymbolFilter(input.value())
                        }
                    )}/>
                </div>
                <div class="m-4">
                    <p class="mb-2 text-gray-500 dark:text-gray-100">{ "Filter by year" }</p>
                    <select onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdateYearFilter(input.value())

                    })}>
                        <option selected=true value="none">{"None"}</option>
                        <option value="2019">{"2019"}</option>
                        <option value="2020">{"2020"}</option>
                        <option value="2021">{"2021"}</option>
                    </select>
                </div>

                { information }
                if let Some(info) = &self.stock_tax_info {
                    <div class="flex flex-wrap">{ for info.iter().map(|f| Self::view_tax(f)) }</div>
                }
                { &self.err }
            </div>
            </main>
            <footer class="bg-gray-100 dark:bg-gray-800 text-center lg:text-left">
                <div class="text-center text-gray-700 dark:text-gray-300 p-4">
                    {"Built by "}
                    <a class="text-gray-800 dark:text-gray-100" href="https://quphase.com/">{"Quphase"}</a>
                    {". Powered by "}
                    <a class="text-gray-800 dark:text-gray-100" href="https://yew.rs/">{"Yew"}</a>
                    {". Open source on "}
                    <a class="text-gray-800 dark:text-gray-100" href="https://github.com/quphase/stocks">{"Github"}</a>
                    {"."}
                </div>
            </footer>
        </>
        }
    }
}

impl Model {
    fn view_tax(data: (&String, &Vec<tax::Information>)) -> Html {
        let (symbol, information) = data;

        let mut sum = 0.;
        for info in information {
            match info {
                tax::Information::PriceDiff(a, _d) => sum += a,
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
                //<div> { format!("{:?}", buys_and_sells) } </div>
                <div class="my-4 mx-4">
                    <h2 class="text-black dark:text-gray-200 text-2xl font-medium leading-tight"> {symbol}</h2>
                    <div class={classes!("bg-gray-200","dark:bg-gray-800", "border-l-8", color_class, "h-96", "overflow-y-auto", "overflow-x-hidden")}>
                        {for information.iter().map(|f| Self::view_information(f))}

                    </div>
                    if let Some(tax::Information::Remaing(q)) = information.last() {
                            <div class="w-fill bg-black text-white"> { format!("Quantity Owned: {}", q) } </div>
                    }

                </div>
        }
    }

    fn view_information(data: &tax::Information) -> Html {
        html! {
            <div class="w-96">
            {
            match data {
                tax::Information::Buy(q, _p, _d) =>
                    html! {
                        <div class="bg-blue-400 dark:bg-blue-800 dark:text-white rounded-md p-1 m-2"> { format!("Buy: {}", q) } </div>
                    },
                tax::Information::Sell(q, _p, _d) =>
                    html! {
                        <div class="bg-indigo-400 dark:bg-indigo-800 dark:text-white rounded p-1 mt-2 mr-2 ml-2"> { format!("Sell: {}", q) } </div>
                    },
                tax::Information::TimeDiff(d) =>
                    html! {
                        <div class="w-80 bg-indigo-200 dark:bg-indigo-600 dark:text-white p-1 ml-8"> { format!("{} days -- {}", d.num_days(), if d.num_days() < 365 { "short-term capital" } else { "long-term capital" }) } </div>
                    },
                 tax::Information::PriceDiff(a, _d) =>
                    html! {
                        if a > &0. {
                            <div class="w-64 bg-green-200 dark:bg-green-600 dark:text-white p-1 ml-24"> { format!("${}", (a*100.).round()/100.)} </div>
                        }
                        else {
                            <div class="w-64 bg-red-200 dark:bg-red-600 dark:text-white p-1 ml-24"> { format!("${}", (a*100.).round()/100.)} </div>
                        }
                    },
                tax::Information::Fees(f) =>
                    html! {
                        <div class="bg-red-100 dark:bg-red-500 dark:text-white w-64 p-1 ml-24"> { format!("-${} (fees)", f) } </div>
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
