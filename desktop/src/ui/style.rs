use stylist::{css, Style};

pub fn icon_wrapper_style() -> Style {
    Style::new(css!(r#"
        display: flex;
        position: relative;
        gap: 1vw;
        flex-direction: row;
        flex-wrap: nowrap;
        justify-content: center;
        align-items: baseline;
    "#)).expect("Error creating style")
}

pub fn icon_style() -> Style {
    Style::new(css!(r#"
        position: relative;
        cursor: pointer;
    "#)).expect("Error creating style")
}

pub fn icon_style_inactive() -> Style {
    Style::new(css!(r#"
        position: relative;
        opacity: 0.5;
        cursor: not-allowed;
    "#)).expect("Error creating style")
}

pub fn icon_description() -> Style {
    Style::new(css!(r#"
        visibility: hidden;
        opacity: 0;
        background: #222;
        color: #fff;
        text-align: center;
        border-radius: 6px;
        padding: 0.5em 1em;
        position: absolute;
        left: 50%;
        top: 110%;
        transform: translateX(-50%);
        transition: opacity 0.3s;
        pointer-events: none;
        white-space: nowrap;
        font-size: 1em;
        user-select: none;
    "#)).expect("Error creating style")
}

pub fn icon_description_visible() -> Style {
    Style::new(css!(r#"
        visibility: visible;
        opacity: 1;
        background: #222;
        color: #fff;
        text-align: center;
        border-radius: 6px;
        padding: 0.5em 1em;
        position: absolute;
        left: 50%;
        top: 110%;
        transform: translateX(-50%);
        transition: opacity 0.3s;
        pointer-events: none;
        white-space: nowrap;
        font-size: 1em;
        user-select: none;
    "#)).expect("Error creating style")
}

pub fn icon_border_style() -> Style {
    Style::new(css!(r#"
        border: 2px dotted #888;
        border-radius: 8px;
        padding: 0.5em;
        cursor: pointer;
    "#)).expect("Error creating style")
}

pub fn back_arrow_style() -> Style {
    Style::new(css!(r#"
        position: absolute;
        opacity: 0.5;
        top: 0px;
        left: 0px;
        width: 2em;
        height: 2em;
        padding: 0.5em;
        cursor: pointer;

        back-arrow-hover:hover & {
            opacity: 1.0;
        }
    "#)).expect("Error creating style")
}

pub fn container_style() -> Style {
    Style::new(css!(r#"
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 10px;
        left: 50%;
        min-width: 60%;
        padding-top: 1rem;
        position: absolute;
        transform: translate(-50%, 0);
        text-align: center;
    "#)).expect("Error creating style")
}

pub fn hide_style() -> Style {
    Style::new(css!(r#"
        opacity: 0;
        visibility: hidden;
        user-select: none;
    "#)).expect("Error creating style")
}

pub fn logo_style() -> Style {
    Style::new(css!(r#"
        width: 40%;
    "#)).expect("Error creating style")
}

pub fn header_style() -> Style {
    Style::new(css!(r#"
        position: relative;
        display: block;
        font-size: 6vh;
        color: white;
        font-weight: bold;
        margin: 0;
        text-align: center;
        user-select: none;
        margin: 1.5vw;
        margin-bottom: 3vw;
    "#)).expect("Error creating style")
}

pub fn subheader_style() -> Style {
    Style::new(css!(r#"
        position: absolute;
        margin-left: 5%;
        font-size: 3vh;
        top: 0.32em;
        left: 100%;
        color: #777;
    "#)).expect("Error creating style")
}

pub fn paragraph_style() -> Style {
    Style::new(css!(r#"
        margin-bottom: 1em;
        margin-top: 1em;
    "#)).expect("Error creating style")
}

pub fn none_style() -> Style {
    Style::new(css!(r#""#)).expect("Error creating style")
}