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
        height: 5em;
    "#)).expect("Error creating style")
}

pub fn icon_style_small() -> Style {
    Style::new(css!(r#"
        position: relative;
        cursor: pointer;
        height: 1em;
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

        &:hover {
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
        text-shadow: 2px 2px 2px black;
    "#)).expect("Error creating style")
}

pub fn subheader_style() -> Style {
    Style::new(css!(r#"
        position: absolute;
        margin-left: 5%;
        font-size: 3vh;
        top: 0.32em;
        left: 100%;
        color: #999;
        text-shadow: 1px 1px 1px black;
    "#)).expect("Error creating style")
}

pub fn paragraph_style() -> Style {
    Style::new(css!(r#"
        margin-bottom: 1.25em;
        text-shadow: 1px 1px 1px black;
    "#)).expect("Error creating style")
}

pub fn none_style() -> Style {
    Style::new(css!(r#""#)).expect("Error creating style")
}

pub fn login_box_style() -> Style {
    Style::new(css!(r#"
        position: absolute;
        white-space: nowrap;
        bottom: 2vh;
        right: 2vh;
        padding: 2vh;
        width: auto;
        max-height: 10vh;
        display: flex;
        font-size: 4vh;
        flex-direction: row;
        flex-wrap: nowrap;
        align-items: center;
        overflow: hidden;
        transition: width 0.3s, background 0.3s, border-radius 0.3s;
        background: rgba(0,0,0,0.2);
        cursor: pointer;
        border-radius: 25px;
    
        .login-name {
            opacity: 0;
            max-width: 0;
            transition: opacity 0.3s, max-width 0.3s, margin-right 0.3s;
            overflow: hidden;
        }
    
        &:hover {
            background: rgba(0,0,0,0.5);
            border-radius: 10px;
        }
        &:hover .login-name {
            opacity: 1;
            max-width: 300px;
            margin-right: 10px;
        }
    "#)).expect("Error creating style")
}

pub fn fancy_link_style() -> Style {
    Style::new(css!(r#"
        cursor: pointer;
        display: inline-block;
        color: #fff;
        margin-left: 0.25em;
        margin-right: 0.25em;

        &:hover {
            animation: slide-gradient 3s linear infinite;
            color: transparent;
            background-image: linear-gradient(45deg,rgb(253, 216, 53),rgb(236, 64, 122), rgb(98, 0, 234), rgb(236, 64, 122), rgb(253, 216, 53));
            -webkit-background-clip: text; 
            background-clip: text;
            cursor: pointer;
            display: inline-block;
            background-position: 100% 0;
            transition: none;
            background-size: 300% 100%;
            text-shadow: none;
        }

        @keyframes slide-gradient {
            0% {
                background-position: 0 0;
            }
            100% {
                background-position: 300% 0;
            }
        }
    "#)).expect("Error creating style")
}

// pub fn text_link_style() -> Style {
//     Style::new(css!(r#"
//         cursor: pointer;
//         display: inline-block;
//         color: #fff;
//     "#)).expect("Error creating style")
// }