use yew::{function_component, html, Html};

use crate::ui::style::*;

#[function_component(TermsComponent)]
pub fn terms_component() -> Html {
    html! {
        <div class={container_style().clone()}>
            <div>
                <h3>{ "By downloading or using this software, you acknowledge and agree to the following terms:" }</h3>
            </div>

            <div class={paragraph_style().clone()}>
                <strong>{ "1. Good Faith Effort:" }</strong>
                { " I have made a reasonable attempt to ensure that this application is safe, functional, and free of major bugs. However, due to the nature of software development, it is impossible to guarantee that the app will always work flawlessly in every environment or scenario." }
            </div>

            <div class={paragraph_style().clone()}>
                <strong>{ "2. No Warranty:" }</strong>
                { " This software is provided 'as-is,' without any warranties or guarantees, express or implied. I do not guarantee that the application will be free from errors, bugs, or interruptions, nor that it will always meet your specific needs or expectations." }
            </div>

            <div class={paragraph_style().clone()}>
                <strong>{ "3. Usage at Your Own Risk:" }</strong>
                { " While I have taken precautions to ensure the app works properly, you are solely responsible for any consequences that arise from using it. I am not liable for any direct, indirect, incidental, special, or consequential damages, including data loss or system issues, that may result from using or not being able to use the application." }
            </div>

            <div class={paragraph_style().clone()}>
                <strong>{ "4. No Responsibility for Third-Party Interactions:" }</strong>
                { " The application may interact with third-party services, websites, or tools. I cannot guarantee the availability or safety of those external services, and I am not responsible for any issues that arise from their use." }
            </div>

            <div class={paragraph_style().clone()}>
                <strong>{ "5. Compliance with Laws:" }</strong>
                { " It is your responsibility to ensure that using this software complies with any relevant laws, regulations, and policies in your region." }
            </div>

            <div class={paragraph_style().clone()}>
                {"If you have any concerns you are welcome to audit the code, build it from" }
                <span>
                    <a class={fancy_link_style().clone()} href={"https://github.com/sheumais/logs/tree/release/desktop"} target="_blank" rel="noopener noreferrer">
                        {"source"}
                    </a>
                </span>
                {"yourself, or contact me through"}
                <span>
                    <a class={fancy_link_style().clone()} href={"https://discord.gg/FjJjXHjUQ4"} target="_blank" rel="noopener noreferrer">
                        {"Discord."}
                    </a>
                </span>
            </div>
        </div>
    }
}