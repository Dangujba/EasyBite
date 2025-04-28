/****************************************************************************************
 * File: easyui.rs (gui)
 * Author: Muhammad Baba Goni
 * Created: March 26, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides a basic Graphical User Interface (GUI) abstraction layer.
 *
 * Responsibilities:
 * -----------------
 * - Render basic GUI elements.
 * - Handle events like clicks, inputs, and forms.
 * - Allow users to build desktop applications visually.
 *
 * Usage:
 * ------
 * Used to create user-friendly applications with graphical interfaces.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use eframe::egui::{
    ColorImage,
    CursorIcon,
    FontData,
    FontDefinitions,
    Id,
    Modal,
    Rect,
    Sense,
    Stroke,
    StrokeKind,
    TextureHandle,
    TextureOptions,
    Ui,
    Vec2,
};

use eframe::egui::{ self, Align2, Color32, FontFamily, FontId, Pos2 };
use egui::text::LayoutJob;
use egui::text::TextFormat;
use egui_extras::{ Column, TableBuilder };
use std::collections::HashSet;
use once_cell::sync::Lazy;
use std::{ fs, thread };
use chrono::{ DateTime, Datelike, Local, NaiveTime, Timelike };
use std::sync::atomic::{ AtomicBool, Ordering };
use std::sync::mpsc::{ Sender, channel, Receiver };
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::{ Duration, Instant };
use uuid::Uuid;
use std::collections::{ VecDeque, HashMap };
use rfd::FileDialog as RfdFileDialog;
use crate::easyui::egui::Context;
use crate::evaluation::Value;
use crate::evaluation::{ Environment, GLOBAL_INTERPRETER };

lazy_static::lazy_static! {
    static ref ICON_CACHE: Lazy<Mutex<HashMap<String, TextureHandle>>> = Lazy::new(|| Mutex::new(HashMap::new()));
    static ref FILE_DIALOG_OPTIONS: Mutex<FileDialogOptions> = Mutex::new(FileDialogOptions::default());
    static ref DIALOG_RESULTS: RwLock<HashMap<String, Option<String>>> = RwLock::new(HashMap::new());
    static ref TIMER_STATES: RwLock<HashMap<String, TimerState>> = RwLock::new(HashMap::new());
    static ref FILTERS: Arc<Mutex<Vec<(String, Vec<String>)>>> = Arc::new(Mutex::new(Vec::new()));
    static ref PAGES_STATES: RwLock<HashMap<String, PagesState>> = RwLock::new(HashMap::new());
    static ref SCROLLBAR_STATES: RwLock<HashMap<String, ScrollBarState>> = RwLock::new(HashMap::new());
    static ref PICTUREBOX_STATES: RwLock<HashMap<String, PictureBoxState>> = RwLock::new(HashMap::new());
    static ref PROGRESSBAR_STATES: RwLock<HashMap<String, ProgressBarState>> = RwLock::new(HashMap::new());
    static ref ACTIVE_FORM_ID: RwLock<Option<String>> = RwLock::new(None);
    static ref FORMS: RwLock<HashMap<String, FormSettings>> = RwLock::new(HashMap::new());
    static ref CONTROLS: RwLock<HashMap<String, ControlSettings>> = RwLock::new(HashMap::new());
    static ref EXIT_SENDER: RwLock<Option<Sender<()>>> = RwLock::new(None);
    static ref TEXT_UPDATE_SENDERS: RwLock<HashMap<String, Sender<(String, String)>>> = RwLock::new(HashMap::new());
    static ref SELECTED_RADIOBOXES: RwLock<HashMap<(String, String), String>> = RwLock::new(HashMap::new());
    static ref COMBOBOX_ITEMS: RwLock<HashMap<String, HashMap<String, (Vec<Arc<Mutex<Value>>>, usize)>>> = RwLock::new(HashMap::new()); 
    static ref LISTBOX_ITEMS: RwLock<HashMap<String, HashMap<String, (Vec<Arc<Mutex<Value>>>, usize)>>> = RwLock::new(HashMap::new());
    static ref FORM_VISIBILITIES: RwLock<HashMap<String, Arc<AtomicBool>>> = RwLock::new(HashMap::new());
    static ref MSGBOXES: std::sync::RwLock<std::collections::HashMap<String, MsgBox>> = 
        std::sync::RwLock::new(std::collections::HashMap::new());
    static ref DATETIMEPICKER_STATES: RwLock<HashMap<String, DateTimePickerState>> = RwLock::new(HashMap::new());
    static ref TIMERPICKER_STATES: Arc<Mutex<HashMap<String, TimerPickerState>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref TABLE_STATES: RwLock<HashMap<String, TableState>> = RwLock::new(HashMap::new());
    static ref MENU_STATES: RwLock<HashMap<String, MenuState>> = RwLock::new(HashMap::new());
    static ref GRIDLAYOUT_STATES: RwLock<HashMap<String, GridLayoutState>> = RwLock::new(HashMap::new());
    static ref HORIZONTALLAYOUT_STATES: RwLock<HashMap<String, HorizontalLayoutState>> = RwLock::new(HashMap::new());
    static ref VERTICALLAYOUT_STATES: RwLock<HashMap<String, VerticalLayoutState>> = RwLock::new(HashMap::new());
    static ref FLOWLAYOUT_STATES: RwLock<HashMap<String, FlowLayoutState>> = RwLock::new(HashMap::new());
    static ref DRAWY_STATES: RwLock<HashMap<String, DrawyState>> = RwLock::new(HashMap::new());
    static ref TREEVIEW_STATES: RwLock<HashMap<String, TreeViewState>> = RwLock::new(HashMap::new());
    static ref SLIDER_STATES: RwLock<HashMap<String, SliderState>> = RwLock::new(HashMap::new());
    static ref NUMBERBOX_STATES: RwLock<HashMap<String, NumberBoxState>> = RwLock::new(HashMap::new());
    static ref RICHTEXT_STATES: RwLock<HashMap<String, RichTextState>> = RwLock::new(HashMap::new());
    static ref TOOLBAR_STATES: RwLock<HashMap<String, ToolbarState>> = RwLock::new(HashMap::new());
    static ref STATUSBAR_STATES: RwLock<HashMap<String, StatusBarState>> = RwLock::new(HashMap::new());
    static ref IMAGEBUTTON_STATES: RwLock<HashMap<String, ImageButtonState>> = RwLock::new(HashMap::new());
    static ref COLORDIALOG_STATES: RwLock<HashMap<String, ColorDialogState>> = RwLock::new(HashMap::new());
}

#[derive(Clone, Debug)]
struct TreeNode {
    id: String, // Unique ID for the node (e.g., UUID)
    text: String, // Display text of the node
    children: Vec<TreeNode>, // Child nodes
    expanded: bool, // Whether the node is expanded
    icon: Option<String>, // Optional path to an icon image
    checkbox: Option<bool>, // Optional checkbox state (None if no checkbox)
}

#[derive(Clone, Debug)]
struct TreeViewState {
    nodes: Vec<TreeNode>,
    selected_node: Option<String>, // Node ID (UUID) of the selected node
}

impl Default for TreeViewState {
    fn default() -> Self {
        TreeViewState {
            nodes: Vec::new(),
            selected_node: None,
        }
    }
}

#[derive(Clone, Debug)]
struct SliderState {
    value: f32,
    min: f32,
    max: f32,
    orientation: SliderOrientation,
}

#[derive(Clone, Debug, PartialEq)]
enum SliderOrientation {
    Horizontal,
    Vertical,
}

impl Default for SliderState {
    fn default() -> Self {
        SliderState {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            orientation: SliderOrientation::Horizontal,
        }
    }
}

#[derive(Clone, Debug)]
struct NumberBoxState {
    value: f64,
    min: f64,
    max: f64,
    increment: f64,
    decimals: usize,
}

impl Default for NumberBoxState {
    fn default() -> Self {
        NumberBoxState {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            increment: 1.0,
            decimals: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct RichTextState {
    text: String,
    formats: Vec<(usize, usize, TextFormat)>, // (start, end, format)
}

impl Default for RichTextState {
    fn default() -> Self {
        RichTextState {
            text: String::new(),
            formats: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct ToolbarItem {
    text: String,
    icon: Option<String>,
    callback: Option<Value>,
}

#[derive(Clone, Debug)]
struct ToolbarState {
    items: Vec<ToolbarItem>,
}

impl Default for ToolbarState {
    fn default() -> Self {
        ToolbarState {
            items: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct StatusBarState {
    text: String,
}

impl Default for StatusBarState {
    fn default() -> Self {
        StatusBarState {
            text: "Ready".to_string(),
        }
    }
}

#[derive(Clone)]
struct ImageButtonState {
    image_path: Option<String>,
    texture_handle: Option<TextureHandle>,
}

impl Default for ImageButtonState {
    fn default() -> Self {
        ImageButtonState {
            image_path: None,
            texture_handle: None,
        }
    }
}

// Manually implementing Debug for ImageButtonState
impl std::fmt::Debug for ImageButtonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageButtonState")
            .field("image_path", &self.image_path)
            .field("texture_handle", &self.texture_handle.is_some())
            .finish()
    }
}

#[derive(Clone, Debug)]
enum ColorDialogMode {
    Basic,
    Advanced,
    Web,
}
#[derive(Clone, Debug)]
struct ColorDialogState {
    selected_color: Color32,
    temp_color: Color32,
    is_open: bool,
    confirmed: bool,
    show_advanced: bool,
    pub show_web_colors: bool,
    mode: ColorDialogMode,
}
impl Default for ColorDialogState {
    fn default() -> Self {
        ColorDialogState {
            selected_color: Color32::WHITE,
            temp_color: Color32::WHITE,
            is_open: false,
            confirmed: false,
            show_advanced: false,
            show_web_colors: false,
            mode: ColorDialogMode::Basic,
        }
    }
}

#[derive(Clone, Debug)]
struct DrawyState {
    position: Pos2, // Current position of the turtle
    heading: f32, // Angle in degrees (0 = east, 90 = north, etc.)
    pen_down: bool, // Whether the pen is down (drawing) or up (not drawing)
    pen_size: f32, // Thickness of the pen
    pen_color: Color32, // Color of the pen
    fill_color: Color32, // Fill color for closed shapes
    speed: f32, // Animation speed (not implemented here, but included for completeness)
    path: VecDeque<(Pos2, Pos2, Stroke)>, // Segments drawn: (start, end, stroke)
    fill_path: Vec<Pos2>, // Points for filling (if filling a shape)
    filling: bool, // Whether currently filling a shape
    pending_moves: VecDeque<(Pos2, Pos2, Stroke)>, // Moves to animate
    animation_progress: f32, // Progress of current move (0.0 to 1.0)
    last_update: f64,
}

impl Default for DrawyState {
    fn default() -> Self {
        DrawyState {
            position: Pos2::ZERO,
            heading: 0.0, // Facing east
            pen_down: true,
            pen_size: 2.0,
            pen_color: Color32::BLACK,
            fill_color: Color32::TRANSPARENT,
            speed: 1.0,
            path: VecDeque::new(),
            fill_path: Vec::new(),
            filling: false,
            pending_moves: VecDeque::new(),
            animation_progress: 0.0,
            last_update: 0.0,
        }
    }
}

#[derive(Default, Clone)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub multiselect: bool,
    pub startingpath: Option<String>,
    pub filters: Option<Vec<(String, Vec<String>)>>, // Changed from filter: Option<String>
}
#[derive(Clone, Copy, PartialEq)]
pub enum MsgBoxIcon {
    None,
    Success, // Checkmark
    Error, // X
    Info, // 'i'
    Warning, // '!'
}

// Enum for button configurations
#[derive(Clone, Copy, PartialEq)]
pub enum MsgBoxButtons {
    Ok,
    YesNo,
    YesNoCancel,
}

pub struct MsgBox {
    id: Id,
    title: Option<String>,
    message: String,
    buttons: MsgBoxButtons,
    icon: MsgBoxIcon,
    modal: Option<Modal>,
    response: Option<String>,
    is_open: bool,
    response_retrieved: bool, // New field to track if response was fetched
    response_sender: Sender<Option<String>>,
}

impl MsgBox {
    pub fn new(
        message: String,
        title: Option<String>,
        buttons: Option<MsgBoxButtons>,
        icon: Option<MsgBoxIcon>
    ) -> Self {
        let (sender, receiver) = channel();
        let id = Id::new(format!("msgbox_{}", uuid::Uuid::new_v4().to_string()));
        MsgBox {
            id,
            title,
            message,
            buttons: buttons.unwrap_or(MsgBoxButtons::Ok),
            icon: icon.unwrap_or(MsgBoxIcon::None),
            response: None,
            is_open: false,
            modal: Some(Modal::new(id)),
            response_retrieved: false, // Initially false
            response_sender: sender,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn get_response(&self) -> Option<&String> {
        self.response.as_ref()
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let icon_text = match self.icon {
            MsgBoxIcon::Success => "✓",
            MsgBoxIcon::Error => "✗",
            MsgBoxIcon::Info => "i",
            MsgBoxIcon::Warning => "!",
            MsgBoxIcon::None => "",
        };

        let icon_color = match self.icon {
            MsgBoxIcon::Success => egui::Color32::GREEN,
            MsgBoxIcon::Error => egui::Color32::RED,
            MsgBoxIcon::Info => egui::Color32::BLUE,
            MsgBoxIcon::Warning => egui::Color32::YELLOW,
            MsgBoxIcon::None => egui::Color32::TRANSPARENT,
        };

        let modal = Modal::new(self.id);
        modal.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(title) = &self.title {
                    ui.heading(title);
                    ui.add_space(8.0);
                }

                ui.horizontal(|ui| {
                    if self.icon != MsgBoxIcon::None {
                        ui.label(egui::RichText::new(icon_text).size(24.0).color(icon_color));
                        ui.add_space(8.0);
                    }
                    ui.label(&self.message);
                });

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    match self.buttons {
                        MsgBoxButtons::Ok => {
                            if ui.button("OK").clicked() {
                                self.response = Some("Ok".to_string());
                                self.is_open = false;
                            }
                        }
                        MsgBoxButtons::YesNo => {
                            if ui.button("Yes").clicked() {
                                self.response = Some("Yes".to_string());
                                self.is_open = false;
                            }
                            if ui.button("No").clicked() {
                                self.response = Some("No".to_string());
                                self.is_open = false;
                            }
                        }
                        MsgBoxButtons::YesNoCancel => {
                            if ui.button("Yes").clicked() {
                                self.response = Some("Yes".to_string());
                                self.is_open = false;
                            }
                            if ui.button("No").clicked() {
                                self.response = Some("No".to_string());
                                self.is_open = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.response = Some("Cancel".to_string());
                                self.is_open = false;
                            }
                        }
                    }
                });
            });
        });
    }
}

#[derive(Clone, Debug)]
struct TimerState {
    enabled: bool, // Whether the timer is running
    interval: u32, // Interval in milliseconds
    last_tick: Option<Instant>, // Time of the last tick
    callback: Option<Value>, // Callback function to execute on tick
}

impl Default for TimerState {
    fn default() -> Self {
        TimerState {
            enabled: false,
            interval: 1000, // Default to 1 second
            last_tick: None,
            callback: None,
        }
    }
}
struct PagesState {
    pages: Vec<Page>, // List of pages
    active_page_index: usize, // Index of the currently active page
    transition_alpha: f32, // Opacity value (0.0 to 1.0) for animation
    in_transition: bool, // Whether a transition is in progress
    transition_start_time: Option<std::time::Instant>, // When the transition started
    use_transition: bool, // New field to enable/disable transitions
}

#[derive(Clone, Debug)]
struct Page {
    title: String, // Display name for the tab
    control_ids: Vec<String>, // IDs of child controls on this page
}

impl Default for PagesState {
    fn default() -> Self {
        PagesState {
            pages: Vec::new(),
            active_page_index: 0,
            transition_alpha: 1.0, // Fully opaque by default
            in_transition: false,
            transition_start_time: None,
            use_transition: true, // Transitions enabled by default
        }
    }
}

#[derive(Clone, Debug)]
struct ScrollBarState {
    value: f32,
    min: f32,
    max: f32,
    large_change: f32,
    small_change: f32,
    orientation: ScrollBarOrientation,
}

#[derive(Clone, Debug, PartialEq)]
enum ScrollBarOrientation {
    Horizontal,
    Vertical,
}

impl Default for ScrollBarState {
    fn default() -> Self {
        ScrollBarState {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            large_change: 10.0,
            small_change: 1.0,
            orientation: ScrollBarOrientation::Vertical,
        }
    }
}

#[derive(Clone)]
struct PictureBoxState {
    image_path: Option<String>,
    size_mode: PictureBoxSizeMode,
    texture_handle: Option<TextureHandle>,
}

#[derive(Clone, Debug, PartialEq)]
enum PictureBoxSizeMode {
    Normal,
    Stretch,
    Zoom,
}

impl Default for PictureBoxState {
    fn default() -> Self {
        PictureBoxState {
            image_path: None,
            size_mode: PictureBoxSizeMode::Normal,
            texture_handle: None,
        }
    }
}

#[derive(Clone, Debug)]
struct ProgressBarState {
    value: f32,
    min: f32,
    max: f32,
    bar_color: Color32,
    style: ProgressBarStyle,
}

#[derive(Clone, Debug, PartialEq)]
enum ProgressBarStyle {
    Solid,
    Marquee,
}

impl Default for ProgressBarState {
    fn default() -> Self {
        ProgressBarState {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            bar_color: Color32::BLUE,
            style: ProgressBarStyle::Solid,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DockStyle {
    None,
    Top,
    Bottom,
    Left,
    Right,
    Fill,
}

#[derive(Clone, Debug)]
struct FormSettings {
    title: String,
    width: f32,
    height: f32,
    visible: bool,
    bg_color: Color32,
    maximized: bool,
    fullscreen: bool,
    startposition: String, // e.g., "centerscreen" or "manual"
    resizable: bool,
    position: Option<(f32, f32)>, // For manual positioning
    border: bool,
    controls_order: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ControlSettings {
    control_type: String,
    form_id: String,
    text: String,
    position: Pos2,
    autosize: bool,
    width: f32,
    height: f32,
    fontname: String,
    fontsize: f32,
    fontweight: String,
    forecolor: Color32,
    backcolor: Color32,
    visible: bool,
    enabled: bool,
    text_alignment: Align2,
    padding: (f32, f32, f32, f32),
    margin: (f32, f32, f32, f32),
    layout_constraint: Option<LayoutConstraint>,
    dock: DockStyle,
    callback: Option<Value>,
    treeview_on_events: HashMap<String, Option<Value>>,
    cursor: String,
    multiline: bool,
    checked: bool,
    group: Option<String>,
    children: Vec<String>, // New: List of child control IDs
    border: bool, // New: Toggle border visibility
    shadow: bool, // New: Toggle shadow for Card
    use_as_default_panel: bool,
    orientation: String,
    border_style: String,
}

impl Default for ControlSettings {
    fn default() -> Self {
        ControlSettings {
            control_type: String::new(),
            form_id: String::new(),
            text: String::new(),
            position: Pos2::new(0.0, 0.0),
            autosize: false,
            width: 0.0,
            height: 0.0,
            fontname: "SegoeUI".to_string(),
            fontsize: 12.0,
            fontweight: "regular".to_string(),
            forecolor: Color32::BLACK,
            backcolor: Color32::TRANSPARENT,
            visible: true,
            enabled: true,
            text_alignment: Align2::LEFT_TOP,
            padding: (0.0, 0.0, 0.0, 0.0),
            margin: (0.0, 0.0, 0.0, 0.0),
            layout_constraint: None,
            dock: DockStyle::None,
            callback: None,
            treeview_on_events: HashMap::new(),
            cursor: "default".to_string(),
            multiline: false,
            checked: false,
            group: None,
            children: Vec::new(), // Default to empty vector
            border: false, // Default to no border
            shadow: false, // Default to no shadow
            use_as_default_panel: false,
            orientation: "horizontal".to_string(),
            border_style: "solid".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct GridLayoutState {
    rows: usize,
    cols: usize,
    cell_width: f32,
    cell_height: f32,
    spacing: f32, // Space between cells
    cell_controls: Vec<Vec<Option<String>>>,
    show_grid_lines: bool,
    line_color: Color32, // New field
    line_thickness: f32, // New field
}

#[derive(Clone, Debug)]
struct HorizontalLayoutState {
    spacing: f32, // Space between children
}

#[derive(Clone, Debug)]
struct VerticalLayoutState {
    spacing: f32, // Space between children
}

#[derive(Clone, Debug)]
struct FlowLayoutState {
    spacing: f32, // Space between children
    wrap: bool, // Whether to wrap to the next line
}

impl Default for GridLayoutState {
    fn default() -> Self {
        GridLayoutState {
            rows: 1,
            cols: 1,
            cell_width: 100.0,
            cell_height: 50.0,
            spacing: 5.0,
            cell_controls: vec![vec![None; 1]; 1],
            show_grid_lines: false,
            line_color: Color32::GRAY, // Default color
            line_thickness: 1.0, // Default thickness
        }
    }
}

impl Default for HorizontalLayoutState {
    fn default() -> Self {
        HorizontalLayoutState { spacing: 5.0 }
    }
}

impl Default for VerticalLayoutState {
    fn default() -> Self {
        VerticalLayoutState { spacing: 5.0 }
    }
}

impl Default for FlowLayoutState {
    fn default() -> Self {
        FlowLayoutState {
            spacing: 5.0,
            wrap: true,
        }
    }
}

#[derive(Clone)]
struct DateTimePickerState {
    selected_datetime: DateTime<Local>,
    format: String,
    is_open: bool, // Add this field
}

impl DateTimePickerState {
    fn new() -> Self {
        Self {
            selected_datetime: Local::now(),
            format: "%Y-%m-%d %H:%M:%S".to_string(),
            is_open: false,
        }
    }
}

#[derive(Clone)]
struct TimerPickerState {
    selected_time: NaiveTime, // The selected time
    format: String, // Format string (e.g., "%H:%M" or "%H:%M:%S")
    is_open: bool, // Whether the picker popup is open
}

#[derive(Clone)]
struct TableState {
    rows: Vec<Vec<String>>,
    headers: Vec<String>,
    row_height: f32,
    scroll_to_row: Option<usize>,
    sort_column: Option<usize>, // Index of the column to sort by
    sort_ascending: bool, // True for ascending, false for descending
}

#[derive(Clone)]
struct MenuItem {
    label: String,
    icon: Option<String>,
    callback: Option<Value>,
    children: Vec<MenuItem>,
    is_separator: bool, // New field for separators
}

#[derive(Clone)]
struct MenuState {
    items: Vec<MenuItem>,
    is_open: bool,
}

#[derive(Clone, Debug)]
enum LayoutConstraint {
    LeftOf {
        target_id: String,
        space: f32,
    },
    RightOf {
        target_id: String,
        space: f32,
    },
    Above {
        target_id: String,
        space: f32,
    },
    Below {
        target_id: String,
        space: f32,
    },
}

pub fn createform(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("createform() expects 3 arguments, got {}", args.len()));
    }

    let title = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("createform() expects a string for title".to_string());
        }
    };
    let width = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("createform() expects a number for width".to_string());
        }
    };
    let height = match &args[2] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("createform() expects a number for height".to_string());
        }
    };

    let form_id = Uuid::new_v4().to_string();
    let settings = FormSettings {
        title,
        width,
        height,
        visible: false, // Initially not visible until showform is called
        bg_color: Color32::from_rgb(230, 230, 230),
        maximized: false,
        fullscreen: false,
        startposition: "centerscreen".to_string(),
        resizable: true,
        position: None,
        border: true,
        controls_order: Vec::new(),
    };

    // Create visibility flag
    let visibility = Arc::new(AtomicBool::new(false));

    // Store form settings and visibility
    let mut forms = FORMS.write().unwrap();
    forms.insert(form_id.clone(), settings);

    // Store visibility in a global map (we’ll add this to lazy_static)
    let mut visibilities = FORM_VISIBILITIES.write().unwrap();
    visibilities.insert(form_id.clone(), visibility);

    println!("Created form {} with visible: false", form_id);
    Ok(Value::FormObject(form_id))
}

pub fn createlabel(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 12 {
        return Err(
            format!(
                "createlabel() expects 2-12 arguments (form_id, text, [x, y, width, height, autosize, fontname, fontsize, fontweight, forecolor, backcolor]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createlabel() expects a Form identifier".to_string());
        }
    };

    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("createlabel() expects a string for text".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let height = args.get(5).map_or(20.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 20.0,
        }
    });
    let autosize = args.get(6).map_or(true, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => true,
        }
    });
    let fontname = args.get(7).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(8).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let fontweight = args.get(9).map_or("regular".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "regular".to_string(),
        }
    });
    let forecolor = args.get(10).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(11).map_or(Color32::TRANSPARENT, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::TRANSPARENT),
            _ => Color32::TRANSPARENT,
        }
    });

    let label_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "label".to_string(),
        form_id: form_id.clone(),
        text,
        position: Pos2::new(x, y),
        autosize,
        width,
        height,
        fontname,
        fontsize,
        fontweight,
        forecolor,
        backcolor,
        visible: true,
        enabled: true,
        text_alignment: Align2::LEFT_TOP,
        padding: (0.0, 0.0, 0.0, 0.0),
        margin: (0.0, 0.0, 0.0, 0.0),
        layout_constraint: None,
        dock: DockStyle::None,
        callback: None,
        treeview_on_events: HashMap::new(),
        cursor: "default".to_string(), // No cursor by default
        multiline: false,
        checked: false,
        group: None,
        children: Vec::new(),
        border: false, // Default to no border
        shadow: false,
        use_as_default_panel: false,
        orientation: "horizontal".to_string(),
        border_style: "solid".to_string(),
    };

    CONTROLS.write().unwrap().insert(label_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(label_id.clone());
    }

    Ok(Value::String(label_id))
}

pub fn createbutton(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 13 {
        return Err(
            format!(
                "createbutton() expects 2-13 arguments (form_id, text, [callback, x, y, width, height, autosize, fontname, fontsize, fontweight, forecolor, backcolor]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createbutton() expects a Form identifier".to_string());
        }
    };

    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("createbutton() expects a string for text".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let callback = args.get(2).cloned();
    let x = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(4).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(5).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let height = args.get(6).map_or(30.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 30.0,
        }
    });
    let autosize = args.get(7).map_or(false, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => false,
        }
    });
    let fontname = args.get(8).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(9).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let fontweight = args.get(10).map_or("regular".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "regular".to_string(),
        }
    });
    let forecolor = args.get(11).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(12).map_or(Color32::LIGHT_GRAY, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::LIGHT_GRAY),
            _ => Color32::LIGHT_GRAY,
        }
    });

    let button_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "button".to_string(),
        form_id: form_id.clone(),
        text,
        position: Pos2::new(x, y),
        autosize,
        width,
        height,
        fontname,
        fontsize,
        fontweight,
        forecolor,
        backcolor,
        visible: true,
        enabled: true,
        text_alignment: Align2::CENTER_CENTER,
        padding: (5.0, 5.0, 5.0, 5.0),
        margin: (0.0, 0.0, 0.0, 0.0),
        layout_constraint: None,
        dock: DockStyle::None,
        callback,
        treeview_on_events: HashMap::new(),
        cursor: "Hand".to_string(), // Hand cursor only for buttons
        multiline: false,
        checked: false,
        group: None,
        children: Vec::new(),
        border: false, // Default to no border
        shadow: false,
        use_as_default_panel: false,
        orientation: "horizontal".to_string(),
        border_style: "solid".to_string(),
    };

    CONTROLS.write().unwrap().insert(button_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(button_id.clone());
    }

    Ok(Value::String(button_id))
}

pub fn createtextbox(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 12 {
        return Err(
            format!(
                "createtextbox() expects 2-12 arguments (form_id, text, [x, y, width, height, autosize, fontname, fontsize, fontweight, forecolor, backcolor]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createtextbox() expects a Form identifier".to_string());
        }
    };

    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("createtextbox() expects a string for text".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(4).map_or(150.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 150.0,
        }
    });
    let height = args.get(5).map_or(30.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 30.0,
        }
    });
    let autosize = args.get(6).map_or(false, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => false,
        }
    });
    let fontname = args.get(7).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(8).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let fontweight = args.get(9).map_or("regular".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "regular".to_string(),
        }
    });
    let forecolor = args.get(10).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(11).map_or(Color32::WHITE, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::WHITE),
            _ => Color32::WHITE,
        }
    });

    let textbox_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "textbox".to_string(),
        form_id: form_id.clone(),
        text, // Use provided text
        position: Pos2::new(x, y),
        autosize,
        width,
        height,
        fontname,
        fontsize,
        fontweight,
        forecolor,
        backcolor,
        visible: true,
        enabled: true,
        text_alignment: Align2::LEFT_CENTER,
        padding: (5.0, 5.0, 5.0, 5.0),
        margin: (0.0, 0.0, 0.0, 0.0),
        layout_constraint: None,
        dock: DockStyle::None,
        callback: None,
        treeview_on_events: HashMap::new(),
        cursor: "default".to_string(),
        multiline: false,
        checked: false,
        group: None,
        children: Vec::new(),
        border: false, // Default to no border
        shadow: false,
        use_as_default_panel: false,
        orientation: "horizontal".to_string(),
        border_style: "solid".to_string(),
    };

    CONTROLS.write().unwrap().insert(textbox_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(textbox_id.clone());
    }

    Ok(Value::String(textbox_id))
}

pub fn createcheckbox(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 13 {
        return Err(
            format!(
                "createcheckbox() expects 2-13 arguments (form_id, text, [x, y, width, height, autosize, fontname, fontsize, fontweight, forecolor, backcolor, checked]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createcheckbox() expects a Form identifier".to_string());
        }
    };

    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("createcheckbox() expects a string for text".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let height = args.get(5).map_or(20.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 20.0,
        }
    });
    let autosize = args.get(6).map_or(true, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => true,
        }
    });
    let fontname = args.get(7).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(8).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let fontweight = args.get(9).map_or("regular".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "regular".to_string(),
        }
    });
    let forecolor = args.get(10).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(11).map_or(Color32::TRANSPARENT, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::TRANSPARENT),
            _ => Color32::TRANSPARENT,
        }
    });
    let checked = args.get(12).map_or(false, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => false,
        }
    });

    let checkbox_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "checkbox".to_string(),
        form_id: form_id.clone(),
        text,
        position: Pos2::new(x, y),
        autosize,
        width,
        height,
        fontname,
        fontsize,
        fontweight,
        forecolor,
        backcolor,
        visible: true,
        enabled: true,
        text_alignment: Align2::LEFT_CENTER,
        padding: (0.0, 0.0, 0.0, 0.0),
        margin: (0.0, 0.0, 0.0, 0.0),
        layout_constraint: None,
        dock: DockStyle::None,
        callback: None,
        treeview_on_events: HashMap::new(),
        cursor: "default".to_string(),
        multiline: false,
        checked,
        group: None,
        children: Vec::new(),
        border: false, // Default to no border
        shadow: false,
        use_as_default_panel: false,
        orientation: "horizontal".to_string(),
        border_style: "solid".to_string(),
    };

    CONTROLS.write().unwrap().insert(checkbox_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(checkbox_id.clone());
    }

    Ok(Value::String(checkbox_id))
}

pub fn createscrollbar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 6 {
        return Err(
            format!(
                "createscrollbar() expects 2-6 arguments (form_id, orientation, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createscrollbar() expects a Form identifier".to_string());
        }
    };

    let orientation = match &args[1] {
        Value::String(s) =>
            match s.to_lowercase().as_str() {
                "horizontal" => ScrollBarOrientation::Horizontal,
                "vertical" => ScrollBarOrientation::Vertical,
                _ => {
                    return Err("createscrollbar() expects 'horizontal' or 'vertical'".to_string());
                }
            }
        _ => {
            return Err("createscrollbar() expects a string for orientation".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args
        .get(4)
        .map_or(if orientation == ScrollBarOrientation::Horizontal { 100.0 } else { 20.0 }, |v| {
            match v {
                Value::Number(n) => *n as f32,
                _ => 20.0,
            }
        });
    let height = args
        .get(5)
        .map_or(if orientation == ScrollBarOrientation::Vertical { 100.0 } else { 20.0 }, |v| {
            match v {
                Value::Number(n) => *n as f32,
                _ => 20.0,
            }
        });

    let scrollbar_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "scrollbar".to_string(),
        form_id,
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::LIGHT_GRAY,
        visible: true,
        enabled: true,
        ..Default::default()
    };

    let form_id_clone = settings.form_id.clone();
    let mut state = ScrollBarState::default();
    state.orientation = orientation;
    CONTROLS.write().unwrap().insert(scrollbar_id.clone(), settings);
    SCROLLBAR_STATES.write().unwrap().insert(scrollbar_id.clone(), state);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id_clone) {
        form.controls_order.push(scrollbar_id.clone());
    }

    Ok(Value::String(scrollbar_id))
}

pub fn createpicturebox(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "createpicturebox() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createpicturebox() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let height = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });

    let picturebox_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "picturebox".to_string(),
        form_id,
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::WHITE,
        visible: true,
        enabled: true,
        ..Default::default()
    };
    let form_id_clone = settings.form_id.clone();

    let state = PictureBoxState::default();
    CONTROLS.write().unwrap().insert(picturebox_id.clone(), settings);
    PICTUREBOX_STATES.write().unwrap().insert(picturebox_id.clone(), state);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id_clone) {
        form.controls_order.push(picturebox_id.clone());
    }

    Ok(Value::String(picturebox_id))
}

pub fn createprogressbar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "createprogressbar() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createprogressbar() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(150.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 150.0,
        }
    });
    let height = args.get(4).map_or(20.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 20.0,
        }
    });

    let progressbar_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "progressbar".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::GRAY,
        visible: true,
        enabled: true,
        ..Default::default()
    };

    let state = ProgressBarState::default();

    let form_id_clone = settings.form_id.clone(); // Clone the form_id before moving settings

    CONTROLS.write().unwrap().insert(progressbar_id.clone(), settings);
    PROGRESSBAR_STATES.write().unwrap().insert(progressbar_id.clone(), state);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id_clone) {
        form.controls_order.push(progressbar_id.clone());
    }

    Ok(Value::String(progressbar_id))
}

pub fn createradiobox(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 3 || args.len() > 14 {
        return Err(
            format!(
                "createradiobox() expects 3-14 arguments (form_id, text, group, [x, y, width, height, autosize, fontname, fontsize, fontweight, forecolor, backcolor, checked]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createradiobox() expects a Form identifier".to_string());
        }
    };

    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("createradiobox() expects a string for text".to_string());
        }
    };

    let group = match &args[2] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("createradiobox() expects a string for group".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(4).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(5).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let height = args.get(6).map_or(20.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 20.0,
        }
    });
    let autosize = args.get(7).map_or(true, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => true,
        }
    });
    let fontname = args.get(8).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(9).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let fontweight = args.get(10).map_or("regular".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "regular".to_string(),
        }
    });
    let forecolor = args.get(11).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(12).map_or(Color32::TRANSPARENT, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::TRANSPARENT),
            _ => Color32::TRANSPARENT,
        }
    });
    let checked = args.get(13).map_or(false, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => false,
        }
    });

    let radiobox_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "radiobox".to_string(),
        form_id: form_id.clone(),
        text,
        position: Pos2::new(x, y),
        autosize,
        width,
        height,
        fontname,
        fontsize,
        fontweight,
        forecolor,
        backcolor,
        visible: true,
        enabled: true,
        text_alignment: Align2::LEFT_CENTER,
        padding: (0.0, 0.0, 0.0, 0.0),
        margin: (0.0, 0.0, 0.0, 0.0),
        layout_constraint: None,
        dock: DockStyle::None,
        callback: None,
        treeview_on_events: HashMap::new(),
        cursor: "default".to_string(),
        multiline: false,
        checked,
        group: Some(group.clone()),
        children: Vec::new(),
        border: false, // Default to no border
        shadow: false,
        use_as_default_panel: false,
        orientation: "horizontal".to_string(),
        border_style: "solid".to_string(),
    };

    if checked {
        let mut selected_radioboxes = SELECTED_RADIOBOXES.write().unwrap();
        selected_radioboxes.insert((form_id.clone(), group), radiobox_id.clone());
    }

    CONTROLS.write().unwrap().insert(radiobox_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(radiobox_id.clone());
    }

    Ok(Value::String(radiobox_id))
}

pub fn creategroupbox(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 7 {
        return Err(
            format!(
                "creategroupbox() expects 2-7 arguments (form_id, title, [x, y, width, height, border]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("creategroupbox() expects a Form identifier".to_string());
        }
    };
    let title = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("creategroupbox() expects a title string".to_string());
        }
    };
    let x = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(4).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(5).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let border = args.get(6).map_or(true, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => true,
        }
    });

    let groupbox_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "groupbox".to_string(),
        form_id: form_id.clone(),
        text: title,
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::TRANSPARENT,
        border,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(groupbox_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(groupbox_id.clone());
    }
    Ok(Value::String(groupbox_id))
}

pub fn createpanel(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 6 {
        return Err(
            format!(
                "createpanel() expects 1-6 arguments (form_id, [x, y, width, height, border]), got {}",
                args.len()
            )
        );
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createpanel() expects a Form identifier".to_string());
        }
    };
    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let border = args.get(5).map_or(false, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => false,
        }
    });

    let panel_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "panel".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::TRANSPARENT,
        border,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(panel_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(panel_id.clone());
    }
    Ok(Value::String(panel_id))
}

pub fn createtimer(args: Vec<Value>) -> Result<Value, String> {
    if args.len() > 2 {
        return Err(
            format!(
                "createtimer() expects 0-2 arguments ([interval, callback]), got {}",
                args.len()
            )
        );
    }

    let interval = args.get(0).map_or(1000, |v| {
        match v {
            Value::Number(n) => *n as u32,
            _ => 1000,
        }
    });

    let callback = args.get(1).cloned();

    let timer_id = Uuid::new_v4().to_string();
    let state = TimerState {
        enabled: false, // Start disabled, like WinForms
        interval,
        last_tick: None,
        callback,
    };

    TIMER_STATES.write().unwrap().insert(timer_id.clone(), state);

    Ok(Value::String(timer_id))
}

pub fn createcard(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "createcard() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createcard() expects a Form identifier".to_string());
        }
    };
    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });

    let card_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "card".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::WHITE, // Default to white for CSS-like card
        shadow: true, // Enable shadow by default
        padding: (10.0, 10.0, 10.0, 10.0), // Default padding
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(card_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(card_id.clone());
    }
    Ok(Value::String(card_id))
}

pub fn createsidebar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 4 {
        return Err(
            format!(
                "createsidebar() expects 1-4 arguments (form_id, [x, width, dock]), got {}",
                args.len()
            )
        );
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createsidebar() expects a Form identifier".to_string());
        }
    };
    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(2).map_or(150.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 150.0,
        }
    });
    let dock = args.get(3).map_or(DockStyle::Left, |v| {
        match v {
            Value::String(s) =>
                match s.as_str() {
                    "left" => DockStyle::Left,
                    "right" => DockStyle::Right,
                    _ => DockStyle::Left,
                }
            _ => DockStyle::Left,
        }
    });

    let sidebar_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "sidebar".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, 0.0), // y=0 to span full height
        width,
        height: 0.0, // Height will be set by form height via docking
        backcolor: Color32::from_gray(240), // Light gray default
        dock,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(sidebar_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(sidebar_id.clone());
    }
    Ok(Value::String(sidebar_id))
}

pub fn createpages(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 7 {
        return Err(
            format!(
                "createpages() expects 1-7 arguments (form_id, [page_titles, default_panel, x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createpages() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let mut page_titles = Vec::new();
    let mut default_panel = false;
    let mut arg_index = 1;

    if let Some(Value::Array(arr)) = args.get(1) {
        for item in arr.iter() {
            if let Value::String(title) = item.lock().unwrap().clone() {
                page_titles.push(title);
            } else {
                return Err(
                    "createpages() expects page_titles to be an array of strings".to_string()
                );
            }
        }
        arg_index = 2;
    }

    if let Some(Value::Bool(b)) = args.get(arg_index) {
        default_panel = *b;
        arg_index += 1;
    }

    let x = args.get(arg_index).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(arg_index + 1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(arg_index + 2).map_or(300.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 300.0,
        }
    });
    let height = args.get(arg_index + 3).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });

    let pages_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "pages".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::from_gray(240),
        visible: true,
        enabled: true,
        children: Vec::new(),
        border: true,
        shadow: false,
        use_as_default_panel: default_panel,
        dock: if default_panel {
            DockStyle::Fill
        } else {
            DockStyle::None
        }, // Set dock to Fill if default_panel
        ..Default::default()
    };

    let mut pages = Vec::new();
    if page_titles.is_empty() {
        pages.push(Page { title: "Page 1".to_string(), control_ids: Vec::new() });
    } else {
        for title in page_titles {
            pages.push(Page { title, control_ids: Vec::new() });
        }
    }

    let state = PagesState {
        pages,
        active_page_index: 0,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(pages_id.clone(), settings);
    PAGES_STATES.write().unwrap().insert(pages_id.clone(), state);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(pages_id.clone());
    }

    Ok(Value::String(pages_id))
}

pub fn createdatetimepicker(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 6 {
        return Err(
            format!(
                "createdatetimepicker() expects 1-6 arguments (form_id, [x, y, width, height, format]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createdatetimepicker() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(4).map_or(30.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 30.0,
        }
    });
    let format = args.get(5).map_or("%Y-%m-%d %H:%M:%S".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "%Y-%m-%d %H:%M:%S".to_string(),
        }
    });

    let datetimepicker_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "datetimepicker".to_string(),
        form_id: form_id.clone(),
        text: Local::now().format(&format).to_string(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::WHITE,
        visible: true,
        enabled: true,
        ..Default::default()
    };

    let state = DateTimePickerState {
        selected_datetime: Local::now(),
        format,
        is_open: false, // Initialize as closed
    };

    CONTROLS.write().unwrap().insert(datetimepicker_id.clone(), settings);
    DATETIMEPICKER_STATES.write().unwrap().insert(datetimepicker_id.clone(), state);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(datetimepicker_id.clone());
    }

    Ok(Value::String(datetimepicker_id))
}

pub fn createtimerpicker(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 6 {
        return Err(
            format!(
                "createtimerpicker() expects 1-6 arguments (form_id, [x, y, width, height, format]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createtimerpicker() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(150.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 150.0,
        }
    });
    let height = args.get(4).map_or(30.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 30.0,
        }
    });
    let format = args.get(5).map_or("%H:%M".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "%H:%M".to_string(), // Default to hours:minutes
        }
    });

    let timerpicker_id = Uuid::new_v4().to_string();
    let initial_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap(); // Default to 00:00:00
    let settings = ControlSettings {
        control_type: "timerpicker".to_string(),
        form_id: form_id.clone(),
        text: initial_time.format(&format).to_string(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::WHITE,
        visible: true,
        enabled: true,
        ..Default::default()
    };

    let state = TimerPickerState {
        selected_time: initial_time,
        format,
        is_open: false,
    };

    CONTROLS.write().unwrap().insert(timerpicker_id.clone(), settings);
    TIMERPICKER_STATES.lock().unwrap().insert(timerpicker_id.clone(), state);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(timerpicker_id.clone());
    }

    Ok(Value::String(timerpicker_id))
}

pub fn createtable(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 10 {
        return Err(
            format!(
                "createtable() expects 1-10 arguments (form_id, [x, y, width, height, fontname, fontsize, forecolor, backcolor, text_alignment]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createtable() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(300.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 300.0,
        }
    });
    let height = args.get(4).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let fontname = args.get(5).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(6).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let forecolor = args.get(7).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(8).map_or(Color32::WHITE, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::WHITE),
            _ => Color32::WHITE,
        }
    });
    let text_alignment = args.get(9).map_or(Align2::LEFT_CENTER, |v| {
        match v {
            Value::String(s) =>
                match s.as_str() {
                    "left_top" => Align2::LEFT_TOP,
                    "left_center" => Align2::LEFT_CENTER,
                    "left_bottom" => Align2::LEFT_BOTTOM,
                    "center_top" => Align2::CENTER_TOP,
                    "center_center" => Align2::CENTER_CENTER,
                    "center_bottom" => Align2::CENTER_BOTTOM,
                    "right_top" => Align2::RIGHT_TOP,
                    "right_center" => Align2::RIGHT_CENTER,
                    "right_bottom" => Align2::RIGHT_BOTTOM,
                    _ => Align2::LEFT_CENTER, // Default if invalid
                }
            _ => Align2::LEFT_CENTER,
        }
    });

    let table_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "table".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        fontname,
        fontsize,
        forecolor,
        backcolor,
        text_alignment, // Use the parsed alignment
        visible: true,
        enabled: true,
        padding: (5.0, 5.0, 5.0, 5.0),
        ..Default::default()
    };

    let state = TableState {
        rows: vec![],
        headers: vec![],
        row_height: 20.0,
        scroll_to_row: None,
        sort_column: None,
        sort_ascending: false,
    };

    CONTROLS.write().unwrap().insert(table_id.clone(), settings);
    TABLE_STATES.write().unwrap().insert(table_id.clone(), state);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(table_id.clone());
    }

    Ok(Value::String(table_id))
}

pub fn createmenu(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 4 {
        return Err(
            format!(
                "createmenu() expects 1-4 arguments (form_id, [x, y, width]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createmenu() expects a Form identifier".to_string());
        }
    };

    // Lock once and reuse the value
    let forms_read = FORMS.read().unwrap();
    let form = match forms_read.get(&form_id) {
        Some(f) => f.clone(),
        None => {
            return Err("Parent form not found".to_string());
        }
    };
    drop(forms_read); // Unlock the read lock early

    let x = match args.get(1) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let y = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let width = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        _ => form.width, // Default to form width if not provided
    };

    let menu_id = Uuid::new_v4().to_string();

    let settings = ControlSettings {
        control_type: "menu".to_string(),
        form_id: form_id.clone(),
        text: "menu".to_string(),
        position: Pos2::new(x, y),
        width,
        height: 30.0,
        backcolor: Color32::LIGHT_GRAY,
        visible: true,
        enabled: true,
        dock: DockStyle::Top,
        ..Default::default()
    };

    let state = MenuState {
        items: vec![],
        is_open: false,
    };

    // Lock and update in one go
    {
        let mut controls = CONTROLS.write().unwrap();
        controls.insert(menu_id.clone(), settings);
    }

    {
        let mut menu_states = MENU_STATES.write().unwrap();
        menu_states.insert(menu_id.clone(), state);
    }

    {
        let mut forms_write = FORMS.write().unwrap();
        if let Some(form) = forms_write.get_mut(&form_id) {
            form.controls_order.push(menu_id.clone());
        }
    }

    Ok(Value::String(menu_id))
}

pub fn createseparator(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 6 {
        return Err(
            format!(
                "createseparator() expects 1-6 arguments (form_id, [orientation, x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createseparator() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let orientation = args.get(1).map_or("horizontal".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "horizontal".to_string(),
        }
    });
    let x = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let height = args.get(5).map_or(2.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 2.0,
        }
    });

    let separator_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "separator".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        orientation,
        backcolor: Color32::GRAY,
        visible: true,
        enabled: true,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(separator_id.clone(), settings);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(separator_id.clone());
    }

    Ok(Value::String(separator_id))
}

pub fn creategridlayout(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 8 {
        return Err(
            "creategridlayout() expects 1-8 arguments (form_id, [rows, cols, x, y, width, height, show_grid_lines])".to_string()
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("creategridlayout() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let rows = args.get(1).map_or(1, |v| {
        match v {
            Value::Number(n) => *n as usize,
            _ => 1,
        }
    });
    let cols = args.get(2).map_or(1, |v| {
        match v {
            Value::Number(n) => *n as usize,
            _ => 1,
        }
    });
    let x = args.get(3).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(4).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(5).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(6).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let show_grid_lines = args.get(7).map_or(false, |v| {
        match v {
            Value::Bool(b) => *b,
            _ => false,
        }
    });

    let grid_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "gridlayout".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::TRANSPARENT,
        visible: true,
        enabled: true,
        children: Vec::new(),
        ..Default::default()
    };

    let state = GridLayoutState {
        rows,
        cols,
        cell_width: width / (cols as f32),
        cell_height: height / (rows as f32),
        spacing: 5.0,
        cell_controls: vec![vec![None; cols]; rows],
        show_grid_lines,
        line_color: Color32::GRAY,
        line_thickness: 1.0,
    };

    CONTROLS.write().unwrap().insert(grid_id.clone(), settings);
    GRIDLAYOUT_STATES.write().unwrap().insert(grid_id.clone(), state);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(grid_id.clone());
    }

    Ok(Value::String(grid_id))
}

pub fn createhorizontallayout(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "createhorizontallayout() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createhorizontallayout() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });

    let hlayout_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "horizontallayout".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::TRANSPARENT,
        visible: true,
        enabled: true,
        children: Vec::new(),
        ..Default::default()
    };

    let state = HorizontalLayoutState::default();

    CONTROLS.write().unwrap().insert(hlayout_id.clone(), settings);
    HORIZONTALLAYOUT_STATES.write().unwrap().insert(hlayout_id.clone(), state);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(hlayout_id.clone());
    }

    Ok(Value::String(hlayout_id))
}

pub fn createverticallayout(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "createverticallayout() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createverticallayout() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });

    let vlayout_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "verticallayout".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::TRANSPARENT,
        visible: true,
        enabled: true,
        children: Vec::new(),
        ..Default::default()
    };

    let state = VerticalLayoutState::default();

    CONTROLS.write().unwrap().insert(vlayout_id.clone(), settings);
    VERTICALLAYOUT_STATES.write().unwrap().insert(vlayout_id.clone(), state);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(vlayout_id.clone());
    }

    Ok(Value::String(vlayout_id))
}

pub fn createflowlayout(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "createflowlayout() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createflowlayout() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(200.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 200.0,
        }
    });
    let height = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });

    let flow_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "flowlayout".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::TRANSPARENT,
        visible: true,
        enabled: true,
        children: Vec::new(),
        ..Default::default()
    };

    let state = FlowLayoutState::default();

    CONTROLS.write().unwrap().insert(flow_id.clone(), settings);
    FLOWLAYOUT_STATES.write().unwrap().insert(flow_id.clone(), state);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(flow_id.clone());
    }

    Ok(Value::String(flow_id))
}

pub fn createshape(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 6 {
        return Err(
            format!(
                "createshape() expects 1-6 arguments (form_id, [x, y, width, height, border_style]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createshape() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = args.get(1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(3).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let height = args.get(4).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let border_style = args.get(5).map_or("solid".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "solid".to_string(),
        }
    });

    let shape_id = uuid::Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "shape".to_string(),
        form_id: form_id.clone(),
        position: Pos2::new(x, y),
        width,
        height,
        border_style,
        ..Default::default()
    };

    // Initialize turtle state with starting position at center
    let turtle_state = DrawyState {
        position: Pos2::new(x + width / 2.0, y + height / 2.0),
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(shape_id.clone(), settings);
    DRAWY_STATES.write().unwrap().insert(shape_id.clone(), turtle_state);
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(shape_id.clone());
    }

    Ok(Value::String(shape_id))
}

pub fn create_treeview(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "create_treeview() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    // Expecting the first argument to be a Form identifier.
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("create_treeview() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    // Extract optional numeric arguments via pattern matching.
    let x = match args.get(1) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };
    let y = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };
    let width = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        _ => 200.0,
    };
    let height = match args.get(4) {
        Some(Value::Number(n)) => *n as f32,
        _ => 200.0,
    };

    let treeview_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "treeview".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::WHITE,
        visible: true,
        enabled: true,
        border: true,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(treeview_id.clone(), settings);
    TREEVIEW_STATES.write().unwrap().insert(treeview_id.clone(), TreeViewState::default());

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(treeview_id.clone());
    }

    Ok(Value::String(treeview_id))
}

pub fn create_slider(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 6 {
        return Err(
            format!(
                "create_slider() expects 2-6 arguments (form_id, orientation, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    // Expect the first argument to be a Form identifier.
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("create_slider() expects a Form identifier".to_string());
        }
    };

    // Extract orientation from the second argument.
    let orientation = match &args[1] {
        Value::String(s) =>
            match s.to_lowercase().as_str() {
                "horizontal" => SliderOrientation::Horizontal,
                "vertical" => SliderOrientation::Vertical,
                _ => {
                    return Err("orientation must be 'horizontal' or 'vertical'".to_string());
                }
            }
        _ => {
            return Err("orientation must be a string".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };
    let y = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };
    let width = match args.get(4) {
        Some(Value::Number(n)) => *n as f32,
        _ => 100.0,
    };
    let height = match args.get(5) {
        Some(Value::Number(n)) => *n as f32,
        _ => 20.0,
    };

    let slider_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "slider".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::LIGHT_GRAY,
        visible: true,
        enabled: true,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(slider_id.clone(), settings);
    SLIDER_STATES.write().unwrap().insert(slider_id.clone(), SliderState {
        orientation,
        ..Default::default()
    });

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(slider_id.clone());
    }

    Ok(Value::String(slider_id))
}

pub fn create_richtext(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "create_richtext() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("create_richtext() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = match args.get(1) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let y = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let width = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        _ => 200.0,
    };

    let height = match args.get(4) {
        Some(Value::Number(n)) => *n as f32,
        _ => 100.0,
    };

    let richtext_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "richtext".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::WHITE,
        visible: true,
        enabled: true,
        border: true,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(richtext_id.clone(), settings);
    RICHTEXT_STATES.write().unwrap().insert(richtext_id.clone(), RichTextState::default());

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(richtext_id.clone());
    }

    Ok(Value::String(richtext_id))
}

pub fn create_toolbar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 4 {
        return Err(
            format!(
                "create_toolbar() expects 1-4 arguments (form_id, [x, y, width]), got {}",
                args.len()
            )
        );
    }

    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("create_toolbar() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let x = match args.get(1) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let y = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let width = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        _ => 300.0,
    };

    let toolbar_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "toolbar".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height: 30.0,
        backcolor: Color32::LIGHT_GRAY,
        visible: true,
        enabled: true,
        dock: DockStyle::Top,
        ..Default::default()
    };

    CONTROLS.write().unwrap().insert(toolbar_id.clone(), settings);
    TOOLBAR_STATES.write().unwrap().insert(toolbar_id.clone(), ToolbarState::default());

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(toolbar_id.clone());
    }

    Ok(Value::String(toolbar_id))
}

pub fn create_statusbar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 4 {
        return Err(
            format!(
                "create_statusbar() expects 1-4 arguments (form_id, [text, x, y, width]), got {}",
                args.len()
            )
        );
    }

    // Extract form_id (first argument)
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("create_statusbar() expects a Form identifier".to_string());
        }
    };

    // Clone form_id before using it in the read lock
    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    // Optional second argument for text
    let text = match args.get(1) {
        Some(Value::String(t)) => t.clone(),
        _ => "Ready".to_string(), // default text if no argument or invalid type
    };

    // Optional x, y, and width arguments
    let x = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0, // default to 0 if no argument or invalid type
    };

    let y = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0, // default to 0 if no argument or invalid type
    };

    let width = match args.get(4) {
        Some(Value::Number(n)) => *n as f32,
        _ => 300.0, // default to 300 if no argument or invalid type
    };

    // Create a unique ID for the status bar
    let statusbar_id = Uuid::new_v4().to_string();

    // Control settings
    let settings = ControlSettings {
        control_type: "statusbar".to_string(),
        form_id: form_id.clone(), // Clone form_id here
        text,
        position: Pos2::new(x, y),
        width,
        height: 20.0,
        backcolor: Color32::BLUE,
        visible: true,
        enabled: true,
        dock: DockStyle::Bottom,
        ..Default::default()
    };

    // Store control settings and initial state
    CONTROLS.write().unwrap().insert(statusbar_id.clone(), settings);
    STATUSBAR_STATES.write().unwrap().insert(statusbar_id.clone(), StatusBarState::default());

    // Update form controls order
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(statusbar_id.clone());
    }

    Ok(Value::String(statusbar_id))
}

pub fn create_imagebutton(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 5 {
        return Err(
            format!(
                "create_imagebutton() expects 1-5 arguments (form_id, [x, y, width, height]), got {}",
                args.len()
            )
        );
    }

    // Extract form_id (first argument)
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("create_imagebutton() expects a Form identifier".to_string());
        }
    };

    // Check if the form exists in FORMS
    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    // Extract x, y, width, and height, defaulting to 0.0 or 50.0 if not provided
    let x = match args.get(1) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let y = match args.get(2) {
        Some(Value::Number(n)) => *n as f32,
        _ => 0.0,
    };

    let width = match args.get(3) {
        Some(Value::Number(n)) => *n as f32,
        _ => 50.0,
    };

    let height = match args.get(4) {
        Some(Value::Number(n)) => *n as f32,
        _ => 50.0,
    };

    // Create a unique ID for the image button
    let imagebutton_id = Uuid::new_v4().to_string();

    // Control settings
    let settings = ControlSettings {
        control_type: "imagebutton".to_string(),
        form_id: form_id.clone(), // Clone here to avoid moving
        text: String::new(),
        position: Pos2::new(x, y),
        width,
        height,
        backcolor: Color32::LIGHT_GRAY,
        visible: true,
        enabled: true,
        ..Default::default()
    };

    // Store control settings and initial state
    CONTROLS.write().unwrap().insert(imagebutton_id.clone(), settings);
    IMAGEBUTTON_STATES.write().unwrap().insert(imagebutton_id.clone(), ImageButtonState::default());

    // Update form controls order
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(imagebutton_id.clone());
    }

    Ok(Value::String(imagebutton_id))
}

pub fn create_colordialog(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!("create_colordialog() expects 1 argument (form_id), got {}", args.len())
        );
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("create_colordialog() expects a Form identifier".to_string());
        }
    };

    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    let colordialog_id = Uuid::new_v4().to_string();
    let settings = ControlSettings {
        control_type: "colordialog".to_string(),
        form_id: form_id.clone(),
        text: String::new(),
        position: Pos2::ZERO,
        width: 0.0,
        height: 0.0,
        visible: true, // Set to true so it’s considered for rendering
        enabled: true,
        ..Default::default()
    };

    let mut controls = CONTROLS.write().unwrap();
    controls.insert(colordialog_id.clone(), settings);

    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(colordialog_id.clone()); // Add to form’s controls
    }

    COLORDIALOG_STATES.write().unwrap().insert(colordialog_id.clone(), ColorDialogState::default());

    Ok(Value::String(colordialog_id))
}

pub fn drawy_forward(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("turtle_forward() expects 2 arguments (shape_id, distance)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a shape identifier".to_string());
        }
    };

    let distance = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Second argument must be a number (distance)".to_string());
        }
    };

    let mut controls = CONTROLS.write().unwrap();
    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if
        let (Some(settings), Some(state)) = (
            controls.get_mut(&shape_id),
            drawy_states.get_mut(&shape_id),
        )
    {
        if settings.control_type != "shape" {
            return Err("Control is not a shape".to_string());
        }

        let angle_rad = state.heading.to_radians();
        let new_x = state.position.x + distance * angle_rad.cos();
        let new_y = state.position.y - distance * angle_rad.sin(); // Subtract because y increases downward in egui
        let new_pos = Pos2::new(new_x, new_y);

        if state.pen_down {
            let stroke = match settings.border_style.as_str() {
                "solid" => Stroke::new(state.pen_size, state.pen_color),
                "dotted" => Stroke::new(state.pen_size, state.pen_color), // Custom rendering needed
                "dashed" => Stroke::new(state.pen_size, state.pen_color), // Custom rendering needed
                _ => Stroke::new(state.pen_size, state.pen_color),
            };
            state.path.push_back((state.position, new_pos, stroke));
        }
        if state.filling {
            state.fill_path.push(new_pos);
        }
        state.position = new_pos;
        Ok(Value::Null)
    } else {
        Err("Shape or turtle state not found".to_string())
    }
}

pub fn drawy_left(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("turtle_left() expects 2 arguments (shape_id, angle)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a shape identifier".to_string());
        }
    };

    let angle = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Second argument must be a number (angle)".to_string());
        }
    };

    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = drawy_states.get_mut(&shape_id) {
        state.heading = (state.heading + angle) % 360.0;
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn drawy_right(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("turtle_right() expects 2 arguments (shape_id, angle)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a shape identifier".to_string());
        }
    };

    let angle = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Second argument must be a number (angle)".to_string());
        }
    };

    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = drawy_states.get_mut(&shape_id) {
        state.heading = (state.heading - angle + 360.0) % 360.0;
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn drawy_penup(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("turtle_penup() expects 1 argument (shape_id)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a shape identifier".to_string());
        }
    };

    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = drawy_states.get_mut(&shape_id) {
        state.pen_down = false;
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn drawy_pendown(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("turtle_pendown() expects 1 argument (shape_id)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a shape identifier".to_string());
        }
    };

    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = drawy_states.get_mut(&shape_id) {
        state.pen_down = true;
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn drawy_pencolor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("turtle_pencolor() expects 2 arguments (shape_id, color)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a shape identifier".to_string());
        }
    };

    let color = match &args[1] {
        Value::String(s) => parse_color(s).unwrap_or(Color32::TRANSPARENT),
        _ => {
            return Err("Second argument must be a color".to_string());
        }
    };

    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = drawy_states.get_mut(&shape_id) {
        state.pen_color = color;
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn drawy_begin_fill(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("turtle_begin_fill() expects 1 argument (shape_id)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a shape identifier".to_string());
        }
    };

    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = drawy_states.get_mut(&shape_id) {
        state.filling = true;
        state.fill_path.clear();
        state.fill_path.push(state.position);
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn drawy_end_fill(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("turtle_end_fill() expects 1 argument (shape_id)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a shape identifier".to_string());
        }
    };

    let mut drawy_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = drawy_states.get_mut(&shape_id) {
        state.filling = false;
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn drawy_goto(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("turtle_goto() expects 3 arguments (shape_id, x, y)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a shape identifier".to_string());
        }
    };

    let x = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Second argument must be a number (x)".to_string());
        }
    };

    let y = match &args[2] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Third argument must be a number (y)".to_string());
        }
    };

    let mut controls = CONTROLS.write().unwrap();
    let mut turtle_states = DRAWY_STATES.write().unwrap();
    if
        let (Some(settings), Some(state)) = (
            controls.get_mut(&shape_id),
            turtle_states.get_mut(&shape_id),
        )
    {
        if settings.control_type != "shape" {
            return Err("Control is not a shape".to_string());
        }

        let new_pos = Pos2::new(x + settings.position.x, y + settings.position.y); // Offset by control position
        if state.pen_down {
            let stroke = Stroke::new(state.pen_size, state.pen_color);
            if state.speed == 0.0 {
                state.path.push_back((state.position, new_pos, stroke));
            } else {
                state.pending_moves.push_back((state.position, new_pos, stroke));
            }
        }
        if state.filling {
            state.fill_path.push(new_pos);
        }
        state.position = new_pos;
        Ok(Value::Null)
    } else {
        Err("Shape or turtle state not found".to_string())
    }
}

pub fn drawy_circle(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            "turtle_circle() expects 2-3 arguments (shape_id, radius, [extent])".to_string()
        );
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a shape identifier".to_string());
        }
    };

    let radius = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Second argument must be a number (radius)".to_string());
        }
    };

    let extent = args.get(2).map_or(360.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 360.0,
        }
    });

    let mut controls = CONTROLS.write().unwrap();
    let mut turtle_states = DRAWY_STATES.write().unwrap();
    if
        let (Some(settings), Some(state)) = (
            controls.get_mut(&shape_id),
            turtle_states.get_mut(&shape_id),
        )
    {
        if settings.control_type != "shape" {
            return Err("Control is not a shape".to_string());
        }

        let steps = (extent.abs() / 5.0).max(10.0) as usize; // At least 10 segments
        let angle_step = extent / (steps as f32);
        let center = Pos2::new(
            state.position.x + radius * state.heading.to_radians().cos(),
            state.position.y - radius * state.heading.to_radians().sin()
        );

        let mut last_pos = state.position;
        for i in 0..=steps {
            let angle = state.heading + angle_step * (i as f32);
            let new_x = center.x - radius * angle.to_radians().cos();
            let new_y = center.y + radius * angle.to_radians().sin();
            let new_pos = Pos2::new(new_x, new_y);

            if state.pen_down {
                let stroke = Stroke::new(state.pen_size, state.pen_color);
                if state.speed == 0.0 {
                    state.path.push_back((last_pos, new_pos, stroke));
                } else {
                    state.pending_moves.push_back((last_pos, new_pos, stroke));
                }
            }
            if state.filling {
                state.fill_path.push(new_pos);
            }
            last_pos = new_pos;
        }
        state.position = last_pos; // Update position to end of circle
        state.heading += extent; // Adjust heading
        Ok(Value::Null)
    } else {
        Err("Shape or turtle state not found".to_string())
    }
}

pub fn drawy_speed(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("turtle_speed() expects 2 arguments (shape_id, speed)".to_string());
    }

    let shape_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a shape identifier".to_string());
        }
    };

    let speed = match &args[1] {
        Value::Number(n) => {
            let s = *n as f32;
            if s < 0.0 || s > 10.0 {
                return Err("Speed must be between 0 and 10".to_string());
            }
            s
        }
        _ => {
            return Err("Second argument must be a number (speed)".to_string());
        }
    };

    let mut turtle_states = DRAWY_STATES.write().unwrap();
    if let Some(state) = turtle_states.get_mut(&shape_id) {
        state.speed = speed;
        Ok(Value::Null)
    } else {
        Err("Turtle state not found".to_string())
    }
}

pub fn runapp(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("runapp() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("runapp() expects a Form identifier".to_string());
        }
    };

    // Set the main form to visible
    {
        let mut forms = FORMS.write().unwrap();
        if let Some(settings) = forms.get_mut(&form_id) {
            settings.visible = true;
            let mut visibilities = FORM_VISIBILITIES.write().unwrap();
            if let Some(visibility) = visibilities.get(&form_id) {
                visibility.store(true, Ordering::Relaxed);
            }
        } else {
            return Err("Form not found".to_string());
        }
    }

    let (exit_tx, exit_rx) = channel();
    {
        let mut exit_sender = EXIT_SENDER.write().unwrap();
        *exit_sender = Some(exit_tx);
    }

    let forms = FORMS.read().unwrap();
    let form_settings = forms.get(&form_id).ok_or("Form not found".to_string())?.clone();

    let mut viewport = egui::ViewportBuilder
        ::default()
        .with_maximized(form_settings.maximized)
        .with_fullscreen(form_settings.fullscreen)
        .with_resizable(form_settings.resizable)
        .with_decorations(form_settings.border);

    if !form_settings.maximized && !form_settings.fullscreen {
        viewport = viewport.with_inner_size([form_settings.width, form_settings.height]);
    }

    if form_settings.startposition == "manual" {
        if let Some((x, y)) = form_settings.position {
            viewport = viewport.with_position([x, y]);
        }
    }

    {
        let mut active_form_id = ACTIVE_FORM_ID.write().unwrap();
        *active_form_id = Some(form_id.clone());
    }

    let options = eframe::NativeOptions {
        centered: true,
        viewport,
        ..Default::default()
    };

    eframe
        ::run_native(
            &form_settings.title,
            options,
            Box::new(|cc| {
                let app = MyApp::new(form_id.clone(), exit_rx, cc);
                Ok(Box::new(app))
            })
        )
        .map_err(|e| format!("Failed to run app: {}", e))?;

    {
        let mut exit_sender = EXIT_SENDER.write().unwrap();
        *exit_sender = None;
    }
    Ok(Value::Null)
}

pub fn createlistbox(args: Vec<Value>) -> Result<Value, String> {
    // Require at least 1 argument: form_id
    if args.is_empty() {
        return Err(
            format!(
                "createlistbox() expects 1-12 arguments (form_id, [items, x, y, width, height, fontname, fontsize, fontweight, forecolor, backcolor, selected_index]), got {}",
                args.len()
            )
        );
    }

    // Extract form_id
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createlistbox() expects a Form identifier".to_string());
        }
    };

    // Verify the form exists
    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    // Make items optional: default to empty vector
    let mut items = Vec::new();
    let mut optional_args_index = 1;
    if args.len() > 1 {
        if let Value::Array(arr) = &args[1] {
            items = arr.clone();
            optional_args_index = 2; // Shift optional args if items are provided
        }
    }

    // Extract optional arguments with adjusted indices
    let x = args.get(optional_args_index).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(optional_args_index + 1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(optional_args_index + 2).map_or(150.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 150.0,
        }
    });
    let height = args.get(optional_args_index + 3).map_or(100.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 100.0,
        }
    });
    let fontname = args.get(optional_args_index + 4).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(optional_args_index + 5).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let fontweight = args.get(optional_args_index + 6).map_or("regular".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "regular".to_string(),
        }
    });
    let forecolor = args.get(optional_args_index + 7).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(optional_args_index + 8).map_or(Color32::WHITE, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::WHITE),
            _ => Color32::WHITE,
        }
    });
    let selected_index = args.get(optional_args_index + 9).map_or(0, |v| {
        match v {
            Value::Number(n) => *n as usize,
            _ => 0,
        }
    });

    // Generate a unique ID for the listbox
    let listbox_id = Uuid::new_v4().to_string();

    // Set initial text based on items and selected_index, or empty if no items
    let text = if !items.is_empty() && selected_index < items.len() {
        items[selected_index].lock().unwrap().to_string()
    } else {
        String::new()
    };

    // Create control settings
    let settings = ControlSettings {
        control_type: "listbox".to_string(),
        form_id: form_id.clone(),
        text,
        position: Pos2::new(x, y),
        autosize: false,
        width,
        height,
        fontname,
        fontsize,
        fontweight,
        forecolor,
        backcolor,
        visible: true,
        enabled: true,
        text_alignment: Align2::LEFT_CENTER,
        padding: (5.0, 5.0, 5.0, 5.0),
        margin: (0.0, 0.0, 0.0, 0.0),
        layout_constraint: None,
        dock: DockStyle::None,
        callback: None,
        treeview_on_events: HashMap::new(),
        cursor: "default".to_string(),
        multiline: false,
        checked: false,
        group: Some(listbox_id.clone()),
        children: items
            .iter()
            .map(|_| String::new())
            .collect(),
        border: true,
        shadow: false,
        use_as_default_panel: false,
        orientation: "horizontal".to_string(),
        border_style: "solid".to_string(),
    };

    // Insert the control into CONTROLS
    let mut controls = CONTROLS.write().unwrap();
    controls.insert(listbox_id.clone(), settings);

    // Update the form's control order
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(listbox_id.clone());
    }

    // Store items and selected_index in LISTBOX_ITEMS
    // Store items in a separate HashMap for ListBox
    let mut listbox_items = HashMap::new();
    listbox_items.insert(listbox_id.clone(), (items, selected_index));
    LISTBOX_ITEMS.write().unwrap().insert(listbox_id.clone(), listbox_items);

    Ok(Value::String(listbox_id))
}

pub fn createcombobox(args: Vec<Value>) -> Result<Value, String> {
    // Require at least 1 argument: form_id
    if args.is_empty() {
        return Err(
            format!(
                "createcombobox() expects 2-13 arguments (form_id, [items, textbox_id, x, y, width, height, fontname, fontsize, fontweight, forecolor, backcolor, selected_index]), got {}",
                args.len()
            )
        );
    }

    // Extract form_id
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("createcombobox() expects a Form identifier".to_string());
        }
    };

    // Verify the form exists
    if !FORMS.read().unwrap().contains_key(&form_id) {
        return Err("Parent form not found".to_string());
    }

    // Make items optional: default to empty vector
    let mut items = Vec::new();
    let mut optional_args_index = 1;
    if args.len() > 1 {
        if let Value::Array(arr) = &args[1] {
            items = arr.clone();
            optional_args_index = 2; // Shift optional args if items are provided
        }
    }

    // Extract optional arguments with adjusted indices
    let textbox_id = args.get(optional_args_index).and_then(|v| {
        match v {
            Value::String(id) => Some(id.clone()),
            _ => None,
        }
    });
    let x = args.get(optional_args_index + 1).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let y = args.get(optional_args_index + 2).map_or(0.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 0.0,
        }
    });
    let width = args.get(optional_args_index + 3).map_or(150.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 150.0,
        }
    });
    let height = args.get(optional_args_index + 4).map_or(30.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 30.0,
        }
    });
    let fontname = args.get(optional_args_index + 5).map_or("Proportional".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "Proportional".to_string(),
        }
    });
    let fontsize = args.get(optional_args_index + 6).map_or(14.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 14.0,
        }
    });
    let fontweight = args.get(optional_args_index + 7).map_or("regular".to_string(), |v| {
        match v {
            Value::String(s) => s.clone(),
            _ => "regular".to_string(),
        }
    });
    let forecolor = args.get(optional_args_index + 8).map_or(Color32::BLACK, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
            _ => Color32::BLACK,
        }
    });
    let backcolor = args.get(optional_args_index + 9).map_or(Color32::WHITE, |v| {
        match v {
            Value::String(s) => parse_color(s).unwrap_or(Color32::WHITE),
            _ => Color32::WHITE,
        }
    });
    let selected_index = args.get(optional_args_index + 10).map_or(0, |v| {
        match v {
            Value::Number(n) => *n as usize,
            _ => 0,
        }
    });

    // Set initial text: prioritize textbox_id if provided, else use selected item if available
    let initial_text = if let Some(textbox_id) = &textbox_id {
        let controls = CONTROLS.read().unwrap();
        controls.get(textbox_id).map_or(String::new(), |ctrl| ctrl.text.clone())
    } else if !items.is_empty() && selected_index < items.len() {
        items[selected_index].lock().unwrap().to_string()
    } else {
        String::new()
    };

    // Generate a unique ID for the combobox
    let combobox_id = Uuid::new_v4().to_string();

    // Create control settings
    let settings = ControlSettings {
        control_type: "combobox".to_string(),
        form_id: form_id.clone(),
        text: initial_text,
        position: Pos2::new(x, y),
        autosize: false,
        width,
        height,
        fontname,
        fontsize,
        fontweight,
        forecolor,
        backcolor,
        visible: true,
        enabled: true,
        text_alignment: Align2::LEFT_CENTER,
        padding: (5.0, 5.0, 5.0, 5.0),
        margin: (0.0, 0.0, 0.0, 0.0),
        layout_constraint: None,
        dock: DockStyle::None,
        callback: None,
        treeview_on_events: HashMap::new(),
        cursor: "default".to_string(),
        multiline: false,
        checked: false,
        group: Some(combobox_id.clone()),
        children: items
            .iter()
            .map(|_| String::new())
            .collect(),
        border: true,
        shadow: false,
        use_as_default_panel: false,
        orientation: "horizontal".to_string(),
        border_style: "solid".to_string(),
    };

    // Insert the control into CONTROLS
    let mut controls = CONTROLS.write().unwrap();
    controls.insert(combobox_id.clone(), settings);

    // Update the form's control order
    let mut forms = FORMS.write().unwrap();
    if let Some(form) = forms.get_mut(&form_id) {
        form.controls_order.push(combobox_id.clone());
    }

    // Store items and selected_index in COMBOBOX_ITEMS
    // Store items in a separate HashMap for ComboBox
    let mut combobox_items = HashMap::new();
    combobox_items.insert(combobox_id.clone(), (items, selected_index));
    COMBOBOX_ITEMS.write().unwrap().insert(combobox_id.clone(), combobox_items);

    Ok(Value::String(combobox_id))
}

struct MyApp {
    form_id: String,
    exit_rx: Arc<Mutex<Receiver<()>>>,
    textbox_texts: HashMap<String, String>,
    text_update_rx: Receiver<(String, String)>,
    shown_viewports: HashSet<String>,
    form_visibility: HashMap<String, Arc<AtomicBool>>, // Visibility state for each form
}

impl MyApp {
    fn new(form_id: String, exit_rx: Receiver<()>, cc: &eframe::CreationContext<'_>) -> Self {
        // Define custom fonts
        let mut fonts = FontDefinitions::default();

        let exe_path = std::env::current_exe().expect("Failed to get executable path");
        let exe_dir = exe_path.parent().expect("Failed to get executable directory");

        let fonts_dir = exe_dir.join("fonts");

        // List of font files to load from the 'fonts' subfolder
        let font_files = [
            ("SegoeUI", "SegoeUI.ttf"),
            ("SegoeUIBold", "SegoeUIBold.ttf"),
            ("SegoeUIItalic", "SegoeUIItalic.ttf"),
            ("SegoeUIBoldItalic", "SegoeUIBoldItalic.ttf"),
            ("SansSerif", "SansSerif.otf"),
            ("Algerian", "algerian.ttf"),
        ];

        for (font_name, file_name) in font_files.iter() {
            let font_path = fonts_dir.join(file_name);
            if font_path.exists() {
                match fs::read(&font_path) {
                    Ok(font_data) => {
                        fonts.font_data.insert(
                            font_name.to_string(),
                            FontData::from_owned(font_data).into()
                        );
                        fonts.families.insert(
                            FontFamily::Name(font_name.to_string().into()),
                            vec![font_name.to_string()]
                        );
                    }
                    Err(e) => eprintln!("Failed to load font {}: {}", font_path.display(), e),
                }
            } /*else {
                eprintln!("Font file not found: {}", font_path.display());
            }*/
        }

        // Set the fonts in the egui context
        cc.egui_ctx.set_fonts(fonts);

        let (text_update_tx, text_update_rx) = channel();
        TEXT_UPDATE_SENDERS.write().unwrap().insert(form_id.clone(), text_update_tx);
        let form_visibility = FORM_VISIBILITIES.read().unwrap().clone();

        Self {
            form_id,
            exit_rx: Arc::new(Mutex::new(exit_rx)),
            textbox_texts: HashMap::new(),
            text_update_rx,
            shown_viewports: HashSet::new(),
            form_visibility,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for exit signal
        if self.exit_rx.lock().unwrap().try_recv().is_ok() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Update textbox texts from the channel
        while let Ok((control_id, new_text)) = self.text_update_rx.try_recv() {
            self.textbox_texts.insert(control_id, new_text);
        }

        // Timer
        {
            let mut timers = TIMER_STATES.write().unwrap();
            let now = Instant::now();
            let mut any_timer_enabled = false;

            for (timer_id, timer) in timers.iter_mut() {
                if timer.enabled {
                    any_timer_enabled = true; // At least one timer is active
                    if let Some(last) = timer.last_tick {
                        let elapsed = now.duration_since(last);
                        if elapsed >= Duration::from_millis(timer.interval as u64) {
                            // Trigger the callback
                            if let Some(callback) = &timer.callback {
                                match callback {
                                    Value::Function { body, closure, object, .. } => {
                                        let body = body.clone();
                                        let closure = closure.clone();
                                        let object = object.clone();
                                        std::thread::spawn(move || {
                                            let mut interpreter =
                                                GLOBAL_INTERPRETER.lock().unwrap();
                                            let mut local_env = Arc::new(
                                                Mutex::new(Environment::new(Some(closure)))
                                            );
                                            if let Some(obj) = object {
                                                let value = obj.lock().unwrap().clone();
                                                local_env
                                                    .lock()
                                                    .unwrap()
                                                    .define("this".to_string(), value);
                                            }
                                            let _ = interpreter.visit_block(&body, &mut local_env);
                                        });
                                    }
                                    _ => {}
                                }
                            }
                            timer.last_tick = Some(now);
                        }
                    }
                }
            }

            // Request continuous repainting if any timer is enabled
            if any_timer_enabled {
                ctx.request_repaint();
            }
        }
        // Clone forms data for this frame
        let forms_data = FORMS.read().unwrap().clone();

        // Render the root form in the central panel
        if let Some(settings) = forms_data.get(&self.form_id) {
            if settings.visible {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Autosize and constraint logic
                    let (autosize_controls, constraint_controls): (Vec<_>, Vec<_>) = {
                        let controls = CONTROLS.read().unwrap();
                        let autosize = controls
                            .iter()
                            .filter(|(_, c)| c.form_id == self.form_id && c.autosize)
                            .map(|(id, c)| (
                                id.clone(),
                                c.text.clone(),
                                c.fontname.clone(),
                                c.fontsize,
                                c.forecolor,
                                c.padding,
                            ))
                            .collect();
                        let constraints = controls
                            .iter()
                            .filter(
                                |(_, c)| c.form_id == self.form_id && c.layout_constraint.is_some()
                            )
                            .map(|(id, c)| (id.clone(), c.layout_constraint.clone().unwrap()))
                            .collect();
                        (autosize, constraints)
                    };

                    let autosize_sizes: HashMap<String, (f32, f32)> = autosize_controls
                        .into_iter()
                        .map(|(id, text, fontname, fontsize, forecolor, padding)| {
                            let control = ControlSettings {
                                fontname,
                                fontsize,
                                forecolor,
                                padding,
                                ..Default::default()
                            };
                            let (text_width, text_height) = ui_text_size(ui, &text, &control);
                            let total_width = text_width + padding.0 + padding.2;
                            let total_height = text_height + padding.1 + padding.3;
                            (id, (total_width, total_height))
                        })
                        .collect();

                    {
                        let mut controls = CONTROLS.write().unwrap();
                        // Apply autosize dimensions
                        for (id, (total_width, total_height)) in &autosize_sizes {
                            if let Some(control) = controls.get_mut(id) {
                                control.width = *total_width;
                                control.height = *total_height;
                            }
                        }
                        // Handle layout constraints
                        let target_data: HashMap<
                            String,
                            (egui::Pos2, f32, f32)
                        > = constraint_controls
                            .iter()
                            .filter_map(|(_, constraint)| {
                                match constraint {
                                    | LayoutConstraint::LeftOf { target_id, .. }
                                    | LayoutConstraint::RightOf { target_id, .. }
                                    | LayoutConstraint::Above { target_id, .. }
                                    | LayoutConstraint::Below { target_id, .. } => {
                                        controls
                                            .get(target_id)
                                            .map(|target| {
                                                (
                                                    target_id.clone(),
                                                    (target.position, target.width, target.height),
                                                )
                                            })
                                    }
                                }
                            })
                            .collect();
                        for (id, constraint) in &constraint_controls {
                            if let Some(control) = controls.get_mut(id) {
                                match constraint {
                                    LayoutConstraint::LeftOf { target_id, space } => {
                                        if
                                            let Some((target_pos, target_width, _)) =
                                                target_data.get(target_id)
                                        {
                                            control.position.x =
                                                target_pos.x - control.width - *space;
                                            control.position.y = target_pos.y;
                                        }
                                    }
                                    LayoutConstraint::RightOf { target_id, space } => {
                                        if
                                            let Some((target_pos, target_width, _)) =
                                                target_data.get(target_id)
                                        {
                                            control.position.x =
                                                target_pos.x + *target_width + *space;
                                            control.position.y = target_pos.y;
                                        }
                                    }
                                    LayoutConstraint::Above { target_id, space } => {
                                        if
                                            let Some((target_pos, _, target_height)) =
                                                target_data.get(target_id)
                                        {
                                            control.position.x = target_pos.x;
                                            control.position.y =
                                                target_pos.y - control.height - *space;
                                        }
                                    }
                                    LayoutConstraint::Below { target_id, space } => {
                                        if
                                            let Some((target_pos, _, target_height)) =
                                                target_data.get(target_id)
                                        {
                                            control.position.x = target_pos.x;
                                            control.position.y =
                                                target_pos.y + *target_height + *space;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Collect controls to render, excluding those assigned to pages
                    let controls_data = {
                        let controls = CONTROLS.read().unwrap();
                        let pages_states = PAGES_STATES.read().unwrap();
                        let container_children: HashSet<String> = controls
                            .values()
                            .filter(|c| is_container(&c.control_type))
                            .flat_map(|c| c.children.clone())
                            .collect();
                        controls
                            .iter()
                            .filter(|(id, c)| {
                                c.form_id == self.form_id &&
                                    c.visible &&
                                    !container_children.contains(*id) &&
                                    !pages_states
                                        .values()
                                        .any(|state| {
                                            state.pages
                                                .iter()
                                                .any(|page| page.control_ids.contains(*id))
                                        })
                            })
                            .map(|(id, c)| (id.clone(), c.clone()))
                            .collect::<Vec<_>>()
                    };

                    // Paint the form background
                    let form_rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(form_rect, 0.0, settings.bg_color);

                    // Render controls not assigned to pages
                    for (control_id, control) in controls_data.iter() {
                        render_control(ui, control_id, control, ctx, &mut self.textbox_texts);
                    }
                });
            }
        }
        // Render All Actives msgbox
        let mut msgboxes = MSGBOXES.write().unwrap();
        let mut to_remove = Vec::new();
        for (id, msgbox) in msgboxes.iter_mut() {
            msgbox.show(ctx);
            // Remove only if closed AND response has been retrieved
            if !msgbox.is_open && msgbox.response_retrieved {
                to_remove.push(id.clone());
            }
        }
        // Clean up closed and retrieved message boxes
        for id in to_remove {
            msgboxes.remove(&id);
        }
        // Handle additional viewports (excluding the root form)
        for (form_id, visibility) in self.form_visibility.iter() {
            if form_id == &self.form_id {
                continue; // Skip the root form
            }

            if visibility.load(Ordering::Relaxed) {
                let form_id_clone = form_id.clone();
                let visibility_clone = visibility.clone();
                let settings = forms_data.get(&form_id_clone).unwrap().clone();
                let textbox_texts_clone = self.textbox_texts.clone();

                ctx.show_viewport_deferred(
                    egui::ViewportId::from_hash_of(&form_id_clone),
                    egui::ViewportBuilder
                        ::default()
                        .with_title(&settings.title)
                        .with_inner_size([settings.width, settings.height])
                        .with_maximized(settings.maximized)
                        .with_fullscreen(settings.fullscreen)
                        .with_resizable(settings.resizable)
                        .with_decorations(settings.border),
                    move |ctx, class| {
                        assert!(
                            class == egui::ViewportClass::Deferred,
                            "This egui backend doesn't support deferred viewports"
                        );

                        egui::CentralPanel::default().show(ctx, |ui| {
                            // Collect controls to render, excluding those assigned to pages
                            let controls_data = {
                                let controls = CONTROLS.read().unwrap();
                                let pages_states = PAGES_STATES.read().unwrap();
                                controls
                                    .iter()
                                    .filter(|(id, c)| {
                                        c.form_id == form_id_clone &&
                                            c.visible &&
                                            !pages_states
                                                .values()
                                                .any(|state| {
                                                    state.pages
                                                        .iter()
                                                        .any(|page| page.control_ids.contains(*id))
                                                })
                                    })
                                    .map(|(id, c)| (id.clone(), c.clone()))
                                    .collect::<Vec<_>>()
                            };

                            // Paint the form background
                            let form_rect = ui.available_rect_before_wrap();
                            ui.painter().rect_filled(form_rect, 0.0, settings.bg_color);

                            // Render controls not assigned to pages
                            let mut textbox_texts = textbox_texts_clone.clone();
                            for (control_id, control) in controls_data.iter() {
                                render_control(ui, control_id, control, ctx, &mut textbox_texts);
                            }
                        });

                        // Handle viewport close
                        if ctx.input(|i| i.viewport().close_requested()) {
                            visibility_clone.store(false, Ordering::Relaxed);
                            let mut forms = FORMS.write().unwrap();
                            if let Some(settings) = forms.get_mut(&form_id_clone) {
                                settings.visible = false;
                            }
                        }
                    }
                );
            }
        }
    }
}

pub fn treeview_add_node(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            format!(
                "treeview_add_node() expects 2-3 arguments (treeview_id, text, [parent_id]), got {}",
                args.len()
            )
        );
    }
    let treeview_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("treeview_id must be a string".to_string());
        }
    };
    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("text must be a string".to_string());
        }
    };
    let parent_id = args.get(2).and_then(|v| {
        match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    });

    let mut states = TREEVIEW_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&treeview_id) {
        let node_id = Uuid::new_v4().to_string();
        let node = TreeNode {
            id: node_id.clone(),
            text,
            children: Vec::new(),
            expanded: false,
            icon: None,
            checkbox: None,
        };
        if let Some(parent_id) = parent_id {
            fn add_to_parent(nodes: &mut Vec<TreeNode>, parent_id: &str, node: TreeNode) -> bool {
                for n in nodes.iter_mut() {
                    if n.id == parent_id {
                        n.children.push(node);
                        return true;
                    }
                    if add_to_parent(&mut n.children, parent_id, node.clone()) {
                        return true;
                    }
                }
                false
            }
            if !add_to_parent(&mut state.nodes, &parent_id, node) {
                return Err("Parent node not found".to_string());
            }
        } else {
            state.nodes.push(node);
        }
        Ok(Value::String(node_id)) // Return the node’s ID
    } else {
        Err("TreeView not found".to_string())
    }
}
pub fn treeview_set_node_icon(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            format!(
                "treeview_set_node_icon() expects 3 arguments (treeview_id, node_id, icon_path), got {}",
                args.len()
            )
        );
    }
    let treeview_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("treeview_id must be a string".to_string());
        }
    };
    let node_id = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("node_id must be a string".to_string());
        }
    };
    let icon_path = match &args[2] {
        Value::String(s) => Some(s.clone()),
        Value::Null => None,
        _ => {
            return Err("icon_path must be a string or null".to_string());
        }
    };
    let mut states = TREEVIEW_STATES.write().unwrap();
    if let Some(tree_state) = states.get_mut(&treeview_id) {
        fn set_icon(nodes: &mut [TreeNode], target: &str, icon: Option<String>) -> bool {
            for node in nodes {
                if node.id == target {
                    node.icon = icon;
                    return true;
                }
                if set_icon(&mut node.children, target, icon.clone()) {
                    return true;
                }
            }
            false
        }
        if set_icon(&mut tree_state.nodes, &node_id, icon_path) {
            Ok(Value::Null)
        } else {
            Err("Node not found".to_string())
        }
    } else {
        Err("TreeView not found".to_string())
    }
}

pub fn treeview_set_node_checkbox(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            format!(
                "treeview_set_node_checkbox() expects 3 arguments (treeview_id, node_id, checked), got {}",
                args.len()
            )
        );
    }
    let treeview_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("treeview_id must be a string".to_string());
        }
    };
    let node_id = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("node_id must be a string".to_string());
        }
    };
    let checked = match &args[2] {
        Value::Bool(b) => Some(*b),
        Value::Null => None,
        _ => {
            return Err("checked must be a boolean or null".to_string());
        }
    };
    let mut states = TREEVIEW_STATES.write().unwrap();
    if let Some(tree_state) = states.get_mut(&treeview_id) {
        fn set_checkbox(nodes: &mut [TreeNode], target: &str, checked: Option<bool>) -> bool {
            for node in nodes {
                if node.id == target {
                    node.checkbox = checked;
                    return true;
                }
                if set_checkbox(&mut node.children, target, checked) {
                    return true;
                }
            }
            false
        }
        if set_checkbox(&mut tree_state.nodes, &node_id, checked) {
            Ok(Value::Null)
        } else {
            Err("Node not found".to_string())
        }
    } else {
        Err("TreeView not found".to_string())
    }
}

pub fn set_treeview_on_event(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            format!(
                "set_treeview_on_event() expects 3 arguments (treeview_id, event_type, callback), got {}",
                args.len()
            )
        );
    }
    // Extract the treeview identifier.
    let treeview_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("treeview_id must be a string".to_string());
        }
    };
    // Extract the event type as a string.
    let event_type = match &args[1] {
        Value::String(e) => e.clone(),
        _ => {
            return Err("event_type must be a string".to_string());
        }
    };
    // Extract the callback (or None if null).
    let callback = match &args[2] {
        Value::Null => None,
        _ => Some(args[2].clone()),
    };

    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&treeview_id) {
        if control.control_type != "treeview" {
            return Err("Control is not a TreeView".to_string());
        }
        // Assume we now have a field in control for storing callbacks per event.
        // For example, treeview_on_events: HashMap<String, Option<Value>>
        control.treeview_on_events.insert(event_type, callback);
        Ok(Value::Null)
    } else {
        Err("TreeView not found".to_string())
    }
}

fn call_on_event(on_event: &Value, event_type: Value, node_value: Value) {
    if let Value::Function { body, closure, object, .. } = on_event {
        let body = body.clone();
        let closure = closure.clone();
        let object = object.clone();
        std::thread::spawn(move || {
            let mut interpreter = GLOBAL_INTERPRETER.lock().unwrap();
            let mut local_env = Arc::new(Mutex::new(Environment::new(Some(closure))));
            if let Some(obj) = object {
                let value = obj.lock().unwrap().clone();
                local_env.lock().unwrap().define("this".to_string(), value);
            }
            local_env.lock().unwrap().define("event_type".to_string(), event_type);
            local_env.lock().unwrap().define("node".to_string(), node_value);
            let _ = interpreter.visit_block(&body, &mut local_env);
        });
    }
}

pub fn treeview_set_selected(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("treeview_set_selected() expects 2 arguments, got {}", args.len()));
    }
    let treeview_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("treeview_id must be a string".to_string());
        }
    };
    let node_id = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("node_id must be a string".to_string());
        }
    };
    let mut states = TREEVIEW_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&treeview_id) {
        state.selected_node = Some(node_id);
        Ok(Value::Null)
    } else {
        Err("TreeView not found".to_string())
    }
}

pub fn treeview_get_selected(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("treeview_get_selected() expects 1 argument, got {}", args.len()));
    }
    let treeview_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("treeview_id must be a string".to_string());
        }
    };
    let states = TREEVIEW_STATES.read().unwrap();
    if let Some(state) = states.get(&treeview_id) {
        Ok(state.selected_node.clone().map(Value::String).unwrap_or(Value::Null))
    } else {
        Err("TreeView not found".to_string())
    }
}

pub fn treeview_get_node(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!(
                "treeview_get_node() expects 2 arguments (treeview_id, node_id), got {}",
                args.len()
            )
        );
    }

    let treeview_id = match &args[0] {
        Value::String(s) => s.clone(),
        other => {
            return Err(
                format!(
                    "treeview_get_node(): first argument must be a string (treeview_id), got {:?}",
                    other
                )
            );
        }
    };

    let node_id = match &args[1] {
        Value::String(s) => s.clone(),
        other => {
            return Err(
                format!(
                    "treeview_get_node(): second argument must be a string (node_id), got {:?}",
                    other
                )
            );
        }
    };

    let states = TREEVIEW_STATES.read().map_err(|_| "Failed to lock TREEVIEW_STATES")?;
    let tree_state = states
        .get(&treeview_id)
        .ok_or_else(|| format!("TreeView not found for id '{}'", treeview_id))?;

    // Recursive search function
    fn find_node<'a>(nodes: &'a [TreeNode], target: &str) -> Option<&'a TreeNode> {
        for node in nodes {
            if node.id == target {
                return Some(node);
            }
            if let Some(found) = find_node(&node.children, target) {
                return Some(found);
            }
        }
        None
    }

    let found_node = find_node(&tree_state.nodes, &node_id).ok_or_else(||
        format!("Node '{}' not found", node_id)
    )?;

    // Wrap the dictionary result in Arc<Mutex<>> if needed
    Ok(node_to_value(found_node))
}

fn node_to_value(node: &TreeNode) -> Value {
    let mut dict: HashMap<String, Arc<Mutex<Value>>> = HashMap::new();

    dict.insert("id".to_string(), Arc::new(Mutex::new(Value::String(node.id.clone()))));
    dict.insert("text".to_string(), Arc::new(Mutex::new(Value::String(node.text.clone()))));
    dict.insert("expanded".to_string(), Arc::new(Mutex::new(Value::Bool(node.expanded))));

    // Insert icon, or Null if not present.
    dict.insert(
        "icon".to_string(),
        Arc::new(
            Mutex::new(match &node.icon {
                Some(icon) => Value::String(icon.clone()),
                None => Value::Null,
            })
        )
    );

    // Insert checkbox state, or Null if not set.
    dict.insert(
        "checkbox".to_string(),
        Arc::new(
            Mutex::new(match node.checkbox {
                Some(b) => Value::Bool(b),
                None => Value::Null,
            })
        )
    );

    // Convert children recursively, wrapping each one in Arc<Mutex<_>>
    let children: Vec<Arc<Mutex<Value>>> = node.children
        .iter()
        .map(|child| Arc::new(Mutex::new(node_to_value(child))))
        .collect();
    dict.insert("children".to_string(), Arc::new(Mutex::new(Value::Array(children))));

    Value::Dictionary(dict)
}

pub fn richtext_set_format(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 5 {
        return Err(
            format!(
                "richtext_set_format() expects 5 arguments (richtext_id, start, end, bold, italic), got {}",
                args.len()
            )
        );
    }

    let richtext_id = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("richtext_id must be a string".to_string());
        }
    };

    let start = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("start must be a number".to_string());
        }
    };

    let end = match &args[2] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("end must be a number".to_string());
        }
    };

    let bold = match &args[3] {
        Value::Bool(b) => *b,
        _ => {
            return Err("bold must be a boolean".to_string());
        }
    };

    let italic = match &args[4] {
        Value::Bool(b) => *b,
        _ => {
            return Err("italic must be a boolean".to_string());
        }
    };

    let mut states = RICHTEXT_STATES.write().unwrap();
    if let Some(state) = states.get_mut(richtext_id) {
        if end > state.text.len() || start > end {
            return Err("Invalid range".to_string());
        }
        state.formats.push((
            start,
            end,
            TextFormat {
                italics: italic,
                font_id: FontId::new(14.0, FontFamily::Proportional),
                ..Default::default()
            },
        ));
        Ok(Value::Null)
    } else {
        Err("RichText not found".to_string())
    }
}

pub fn toolbar_add_item(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 4 {
        return Err(
            format!(
                "toolbar_add_item() expects 2-4 arguments (toolbar_id, text, [icon, callback]), got {}",
                args.len()
            )
        );
    }

    // Matching toolbar_id and text to string
    let toolbar_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("toolbar_id must be a string".to_string());
        }
    };

    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("text must be a string".to_string());
        }
    };

    // Handling optional icon
    let icon = match args.get(2) {
        Some(Value::String(s)) => Some(s.clone()),
        _ => None,
    };

    // Handling optional callback
    let callback = args.get(3).cloned();

    // Writing to toolbar state
    let mut states = TOOLBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&toolbar_id) {
        state.items.push(ToolbarItem {
            text,
            icon,
            callback,
        });
        Ok(Value::Null)
    } else {
        Err("Toolbar not found".to_string())
    }
}

pub fn colordialog_show(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("colordialog_show() expects 1 argument, got {}", args.len()));
    }
    let colordialog_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("colordialog_id must be a string".into());
        }
    };
    let mut states = COLORDIALOG_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&colordialog_id) {
        state.is_open = true;
        Ok(Value::Null)
    } else {
        Err("ColorDialog not found".to_string())
    }
}

pub fn colordialog_get_color(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 2 {
        return Err(format!("colordialog_get_color() expects 1 or 2 arguments, got {}", args.len()));
    }

    let colordialog_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("colordialog_id must be a string".into());
        }
    };

    let format = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.to_lowercase(),
            _ => {
                return Err("Format argument must be a string (\"rgb\" or \"hex\")".into());
            }
        }
    } else {
        "rgb".to_string()
    };

    let states = COLORDIALOG_STATES.read().unwrap();
    if let Some(state) = states.get(&colordialog_id) {
        let c = state.selected_color;

        let result = match format.as_str() {
            "hex" => format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b()),
            "rgb" => format!("{},{},{}", c.r(), c.g(), c.b()),
            _ => {
                return Err("Invalid format. Use \"rgb\" or \"hex\".".into());
            }
        };

        Ok(Value::String(result))
    } else {
        Err("ColorDialog not found".to_string())
    }
}

pub fn set_maximized(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_maximized() expects 2 arguments, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("set_maximized() expects a Form identifier".to_string());
        }
    };
    let maximized = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_maximized() expects a boolean".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.maximized = maximized;
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn get_maximized(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_maximized() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("get_maximized() expects a Form identifier".to_string());
        }
    };

    let forms = FORMS.read().unwrap();
    if let Some(settings) = forms.get(&form_id) {
        Ok(Value::Bool(settings.maximized))
    } else {
        Err("Form not found".to_string())
    }
}

pub fn set_fullscreen(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_fullscreen() expects 2 arguments, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("set_fullscreen() expects a Form identifier".to_string());
        }
    };
    let fullscreen = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_fullscreen() expects a boolean".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.fullscreen = fullscreen;
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn get_fullscreen(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_fullscreen() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("get_fullscreen() expects a Form identifier".to_string());
        }
    };

    let forms = FORMS.read().unwrap();
    if let Some(settings) = forms.get(&form_id) {
        Ok(Value::Bool(settings.fullscreen))
    } else {
        Err("Form not found".to_string())
    }
}

pub fn set_startposition(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_startposition() expects 2 arguments, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("set_startposition() expects a Form identifier".to_string());
        }
    };
    let startposition = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_startposition() expects a string".to_string());
        }
    };

    if startposition != "centerscreen" && startposition != "manual" {
        return Err("startposition must be 'centerscreen' or 'manual'".to_string());
    }

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.startposition = startposition;
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn get_startposition(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_startposition() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("get_startposition() expects a Form identifier".to_string());
        }
    };

    let forms = FORMS.read().unwrap();
    if let Some(settings) = forms.get(&form_id) {
        Ok(Value::String(settings.startposition.clone()))
    } else {
        Err("Form not found".to_string())
    }
}

pub fn set_position(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("set_position() expects 3 arguments, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("set_position() expects a Form identifier".to_string());
        }
    };
    let x = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_position() expects a number for x".to_string());
        }
    };
    let y = match &args[2] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_position() expects a number for y".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.position = Some((x, y));
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn set_resizable(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_resizable() expects 2 arguments, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("set_resizable() expects a Form identifier".to_string());
        }
    };
    let resizable = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_resizable() expects a boolean".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.resizable = resizable;
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn get_resizable(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_resizable() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("get_resizable() expects a Form identifier".to_string());
        }
    };

    let forms = FORMS.read().unwrap();
    if let Some(settings) = forms.get(&form_id) {
        Ok(Value::Bool(settings.resizable))
    } else {
        Err("Form not found".to_string())
    }
}

pub fn set_border(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_border() expects 2 arguments, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("set_border() expects a Form identifier".to_string());
        }
    };
    let border = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_border() expects a boolean".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.border = border;
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn get_border(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_border() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("get_border() expects a Form identifier".to_string());
        }
    };

    let forms = FORMS.read().unwrap();
    if let Some(settings) = forms.get(&form_id) {
        Ok(Value::Bool(settings.border))
    } else {
        Err("Form not found".to_string())
    }
}

pub fn show_form(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("show_form() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("show_form() expects a Form identifier".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.visible = true;
        let visibilities = FORM_VISIBILITIES.read().unwrap();
        if let Some(visibility) = visibilities.get(&form_id) {
            visibility.store(true, Ordering::Relaxed);
        }
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn hide_form(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("hide_form() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("hide_form() expects a Form identifier".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.visible = false;
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn exit(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("exit() expects 0 arguments, got {}", args.len()));
    }

    let exit_sender = EXIT_SENDER.read().unwrap();
    if let Some(sender) = &*exit_sender {
        sender.send(()).map_err(|e| format!("Failed to send exit signal: {}", e))?;
        Ok(Value::Null)
    } else {
        Err("Application not running".to_string())
    }
}

pub fn restore_form(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("restore_form() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("restore_form() expects a Form identifier".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if let Some(settings) = forms.get_mut(&form_id) {
        settings.maximized = false;
        settings.fullscreen = false;
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

pub fn close_form(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("close_form() expects 1 argument, got {}", args.len()));
    }
    let form_id = match &args[0] {
        Value::FormObject(id) => id.clone(),
        _ => {
            return Err("close_form() expects a Form identifier".to_string());
        }
    };

    let mut forms = FORMS.write().unwrap();
    if forms.remove(&form_id).is_some() {
        Ok(Value::Null)
    } else {
        Err("Form not found".to_string())
    }
}

macro_rules! control_setter_f32 {
    ($name:ident, $field:ident) => {
        pub fn $name(args: Vec<Value>) -> Result<Value, String> {
            if args.len() != 2 {
                return Err(format!("{}() expects 2 arguments, got {}", stringify!($name), args.len()));
            }
            let control_id = match &args[0] {
                Value::String(id) => id.clone(),
                _ => return Err(format!("{}() expects a control identifier", stringify!($name))),
            };
            let value = match &args[1] {
                Value::Number(v) => *v as f32,
                _ => return Err(format!("{}() expects a number", stringify!($name))),
            };
            let mut controls = CONTROLS.write().unwrap();
            if let Some(control) = controls.get_mut(&control_id) {
                control.$field = value;
                Ok(Value::Null)
            } else {
                Err("Control not found".to_string())
            }
        }
    };
}

macro_rules! control_setter {
    ($name:ident, $field:ident, $type:ty, $variant:ident) => {
        pub fn $name(args: Vec<Value>) -> Result<Value, String> {
            if args.len() != 2 {
                return Err(format!("{}() expects 2 arguments, got {}", stringify!($name), args.len()));
            }
            let control_id = match &args[0] {
                Value::String(id) => id.clone(),
                _ => return Err(format!("{}() expects a control identifier", stringify!($name))),
            };
            let value = match &args[1] {
                Value::$variant(v) => v.clone(),
                _ => return Err(format!("{}() expects a {}", stringify!($name), stringify!($type))),
            };
            let mut controls = CONTROLS.write().unwrap();
            if let Some(control) = controls.get_mut(&control_id) {
                control.$field = value;
                Ok(Value::Null)
            } else {
                Err("Control not found".to_string())
            }
        }
    };
}

macro_rules! control_getter {
    ($name:ident, $field:ident, $type:ty, $variant:ident) => {
        pub fn $name(args: Vec<Value>) -> Result<Value, String> {
            if args.len() != 1 {
                return Err(format!("{}() expects 1 argument, got {}", stringify!($name), args.len()));
            }
            let control_id = match &args[0] {
                Value::String(id) => id.clone(),
                _ => return Err(format!("{}() expects a control identifier", stringify!($name))),
            };
            let controls = CONTROLS.read().unwrap();
            if let Some(control) = controls.get(&control_id) {
                Ok(Value::$variant(control.$field.clone().into()))
            } else {
                Err("Control not found".to_string())
            }
        }
    };
}

control_setter!(set_autosize, autosize, bool, Bool);
control_getter!(get_autosize, autosize, bool, Bool);

control_setter_f32!(set_width, width);
control_getter!(get_width, width, f32, Number);

control_setter_f32!(set_height, height);
control_getter!(get_height, height, f32, Number);

pub fn set_x(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_x() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_x() expects a control identifier".to_string());
        }
    };
    let x = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_x() expects a number".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.position.x = x;
        control.layout_constraint = None; // Reset any layout constraint
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn get_x(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_x() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_x() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        Ok(Value::Number(control.position.x as f64))
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_y(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_y() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_y() expects a control identifier".to_string());
        }
    };
    let y = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_y() expects a number".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.position.y = y;
        control.layout_constraint = None; // Reset any layout constraint
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn get_y(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_y() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_y() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        Ok(Value::Number(control.position.y as f64))
    } else {
        Err("Control not found".to_string())
    }
}

control_setter!(set_fontname, fontname, String, String);
control_getter!(get_fontname, fontname, String, String);

control_setter_f32!(set_fontsize, fontsize);
control_getter!(get_fontsize, fontsize, f32, Number);

control_setter!(set_fontweight, fontweight, String, String);
control_getter!(get_fontweight, fontweight, String, String);

pub fn set_forecolor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_forecolor() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_forecolor() expects a control identifier".to_string());
        }
    };
    let color = match &args[1] {
        Value::String(s) => parse_color(s).unwrap_or(Color32::BLACK),
        _ => {
            return Err("set_forecolor() expects a color string (e.g., '255,0,0')".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.forecolor = color;
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn get_forecolor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_forecolor() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_forecolor() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        let c = control.forecolor;
        Ok(Value::String(format!("{},{},{}", c.r(), c.g(), c.b())))
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_backcolor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_backcolor() expects 2 arguments, got {}", args.len()));
    }
    let id = match &args[0] {
        Value::String(id) => id.clone(),
        Value::FormObject(id) => id.clone(), // Handle form IDs if passed as FormObject
        _ => {
            return Err("set_backcolor() expects a control or form identifier".to_string());
        }
    };
    let color = match &args[1] {
        Value::String(s) => parse_color(s).unwrap_or(Color32::TRANSPARENT),
        _ => {
            return Err("set_backcolor() expects a color string (e.g., '255,0,0')".to_string());
        }
    };

    // Check if the ID corresponds to a form
    {
        let mut forms = FORMS.write().unwrap();
        if let Some(form) = forms.get_mut(&id) {
            form.bg_color = color;
            return Ok(Value::Null);
        }
    } // Release the write lock on FORMS

    // If not a form, check if it’s a control
    {
        let mut controls = CONTROLS.write().unwrap();
        if let Some(control) = controls.get_mut(&id) {
            control.backcolor = color;
            return Ok(Value::Null);
        }
    } // Release the write lock on CONTROLS

    // If neither a form nor a control is found
    Err("Form or control not found".to_string())
}

pub fn get_backcolor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_backcolor() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_backcolor() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        let c = control.backcolor;
        Ok(Value::String(format!("{},{},{}", c.r(), c.g(), c.b())))
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_text_alignment(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_text_alignment() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_text_alignment() expects a control identifier".to_string());
        }
    };
    let alignment = match &args[1] {
        Value::String(s) =>
            match s.as_str() {
                "topleft" => Align2::LEFT_TOP,
                "topcenter" => Align2::CENTER_TOP,
                "topright" => Align2::RIGHT_TOP,
                "middleleft" => Align2::LEFT_CENTER,
                "center" => Align2::CENTER_CENTER,
                "middleright" => Align2::RIGHT_CENTER,
                "bottomleft" => Align2::LEFT_BOTTOM,
                "bottomcenter" => Align2::CENTER_BOTTOM,
                "bottomright" => Align2::RIGHT_BOTTOM,
                _ => {
                    return Err(
                        "Invalid alignment: use topleft, topcenter, topright, middleleft, center, middleright, bottomleft, bottomcenter, bottomright".to_string()
                    );
                }
            }
        _ => {
            return Err("set_text_alignment() expects a string".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.text_alignment = alignment;
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_left(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            format!(
                "set_left() expects 2-3 arguments (control_id, target_id, [space]), got {}",
                args.len()
            )
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_left() expects a control identifier".to_string());
        }
    };
    let target_id = match &args[1] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_left() expects a target control identifier".to_string());
        }
    };
    let space = args.get(2).map_or(1.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 1.0,
        }
    });

    let mut controls = CONTROLS.write().unwrap();
    if !controls.contains_key(&target_id) {
        return Err("Target control not found".to_string());
    }
    if let Some(control) = controls.get_mut(&control_id) {
        control.layout_constraint = Some(LayoutConstraint::LeftOf { target_id, space });
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_above(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            format!(
                "set_above() expects 2-3 arguments (control_id, target_id, [space]), got {}",
                args.len()
            )
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_above() expects a control identifier".to_string());
        }
    };
    let target_id = match &args[1] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_above() expects a target control identifier".to_string());
        }
    };
    let space = args.get(2).map_or(1.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 1.0,
        }
    });

    let mut controls = CONTROLS.write().unwrap();
    if !controls.contains_key(&target_id) {
        return Err("Target control not found".to_string());
    }
    if let Some(control) = controls.get_mut(&control_id) {
        control.layout_constraint = Some(LayoutConstraint::Above { target_id, space });
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_below(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            format!(
                "set_below() expects 2-3 arguments (control_id, target_id, [space]), got {}",
                args.len()
            )
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_below() expects a control identifier".to_string());
        }
    };
    let target_id = match &args[1] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_below() expects a target control identifier".to_string());
        }
    };
    let space = args.get(2).map_or(1.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 1.0,
        }
    });

    let mut controls = CONTROLS.write().unwrap();
    if !controls.contains_key(&target_id) {
        return Err("Target control not found".to_string());
    }
    if let Some(control) = controls.get_mut(&control_id) {
        control.layout_constraint = Some(LayoutConstraint::Below { target_id, space });
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_right(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            format!(
                "set_right() expects 2-3 arguments (control_id, target_id, [space]), got {}",
                args.len()
            )
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_right() expects a control identifier".to_string());
        }
    };
    let target_id = match &args[1] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_right() expects a target control identifier".to_string());
        }
    };
    let space = args.get(2).map_or(1.0, |v| {
        match v {
            Value::Number(n) => *n as f32,
            _ => 1.0,
        }
    });

    let mut controls = CONTROLS.write().unwrap();
    if !controls.contains_key(&target_id) {
        return Err("Target control not found".to_string());
    }
    if let Some(control) = controls.get_mut(&control_id) {
        control.layout_constraint = Some(LayoutConstraint::RightOf { target_id, space });
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_padding(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 5 {
        return Err(
            format!(
                "set_padding() expects 5 arguments (control_id, left, top, right, bottom), got {}",
                args.len()
            )
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_padding() expects a control identifier".to_string());
        }
    };
    let left = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_padding() expects a number for left".to_string());
        }
    };
    let top = match &args[2] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_padding() expects a number for top".to_string());
        }
    };
    let right = match &args[3] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_padding() expects a number for right".to_string());
        }
    };
    let bottom = match &args[4] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_padding() expects a number for bottom".to_string());
        }
    };

    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.padding = (left, top, right, bottom);
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_margin(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 5 {
        return Err(
            format!(
                "set_margin() expects 5 arguments (control_id, left, top, right, bottom), got {}",
                args.len()
            )
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_margin() expects a control identifier".to_string());
        }
    };
    let left = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_margin() expects a number for left".to_string());
        }
    };
    let top = match &args[2] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_margin() expects a number for top".to_string());
        }
    };
    let right = match &args[3] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_margin() expects a number for right".to_string());
        }
    };
    let bottom = match &args[4] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("set_margin() expects a number for bottom".to_string());
        }
    };

    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.margin = (left, top, right, bottom);
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn settext(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("settext() expects 2 arguments (id, text), got {}", args.len()));
    }

    let text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("settext() expects a string for text".to_string());
        }
    };

    match &args[0] {
        // Case: Control ID (String)
        Value::String(control_id) => {
            let mut controls = CONTROLS.write().unwrap();
            if let Some(control) = controls.get_mut(control_id) {
                control.text = text.clone();

                match control.control_type.as_str() {
                    "textbox" => {
                        let form_id = control.form_id.clone();
                        if let Some(sender) = TEXT_UPDATE_SENDERS.read().unwrap().get(&form_id) {
                            sender
                                .send((control_id.clone(), text.clone()))
                                .map_err(|e| format!("Failed to send text update: {}", e))?;
                        }
                    }
                    "richtext" => {
                        let mut richtext_states = RICHTEXT_STATES.write().unwrap();
                        if let Some(state) = richtext_states.get_mut(control_id) {
                            state.text = text.clone();
                        }
                    }
                    "statusbar" => {
                        let mut statusbar_states = STATUSBAR_STATES.write().unwrap();
                        if let Some(state) = statusbar_states.get_mut(control_id) {
                            state.text = text.clone();
                        }
                    }
                    _ => {}
                }

                Ok(Value::Null)
            } else {
                Err("Control not found".to_string())
            }
        }

        // Case: FormObject ID
        Value::FormObject(form_id) => {
            let mut forms = FORMS.write().unwrap();
            if let Some(form) = forms.get_mut(form_id) {
                form.title = text;
                Ok(Value::Null)
            } else {
                Err("Form not found".to_string())
            }
        }

        _ =>
            Err(
                "settext() expects a control identifier (string) or form identifier (FormObject)".to_string()
            ),
    }
}

pub fn gettext(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("gettext() expects 1 argument (id), got {}", args.len()));
    }

    match &args[0] {
        Value::String(control_id) => {
            // First, check control registry
            let controls = CONTROLS.read().unwrap();
            if let Some(control) = controls.get(control_id) {
                match control.control_type.as_str() {
                    "richtext" => {
                        let states = RICHTEXT_STATES.read().unwrap();
                        if let Some(state) = states.get(control_id) {
                            return Ok(Value::String(state.text.clone()));
                        }
                    }
                    "statusbar" => {
                        let states = STATUSBAR_STATES.read().unwrap();
                        if let Some(state) = states.get(control_id) {
                            return Ok(Value::String(state.text.clone()));
                        }
                    }
                    _ => {
                        return Ok(Value::String(control.text.clone()));
                    }
                }
                // If control type found but not in states
                Err(format!("{} not found in its state store", control.control_type))
            } else {
                Err("Control not found".to_string())
            }
        }

        Value::FormObject(form_id) => {
            let forms = FORMS.read().unwrap();
            if let Some(form) = forms.get(form_id) {
                Ok(Value::String(form.title.clone()))
            } else {
                Err("Form not found".to_string())
            }
        }

        _ =>
            Err(
                "gettext() expects a control identifier (string) or form identifier (FormObject)".to_string()
            ),
    }
}

fn parse_color(color: &str) -> Option<Color32> {
    // Handle color names
    match color.to_lowercase().as_str() {
        "red" => {
            return Some(Color32::from_rgb(255, 0, 0));
        }
        "green" => {
            return Some(Color32::from_rgb(0, 255, 0));
        }
        "blue" => {
            return Some(Color32::from_rgb(0, 0, 255));
        }
        "white" => {
            return Some(Color32::from_rgb(255, 255, 255));
        }
        "black" => {
            return Some(Color32::from_rgb(0, 0, 0));
        }
        "yellow" => {
            return Some(Color32::from_rgb(255, 255, 0));
        }
        "cyan" => {
            return Some(Color32::from_rgb(0, 255, 255));
        }
        "magenta" => {
            return Some(Color32::from_rgb(255, 0, 255));
        }
        _ => {}
    }

    // Handle hexadecimal color codes
    if color.starts_with('#') || color.starts_with("0x") {
        let hex_str = if color.starts_with('#') { &color[1..] } else { &color[2..] };
        if hex_str.len() == 6 {
            if let Ok(value) = u32::from_str_radix(hex_str, 16) {
                let r = ((value >> 16) & 0xff) as u8;
                let g = ((value >> 8) & 0xff) as u8;
                let b = (value & 0xff) as u8;
                return Some(Color32::from_rgb(r, g, b));
            }
        }
    }

    // Handle RGB strings (original functionality)
    let parts: Vec<&str> = color.split(',').collect();
    if parts.len() == 3 {
        if
            let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].trim().parse::<u8>(),
                parts[1].trim().parse::<u8>(),
                parts[2].trim().parse::<u8>(),
            )
        {
            return Some(Color32::from_rgb(r, g, b));
        }
    }

    // Return None if no format matches
    None
}

fn get_font_family(fontname: &str) -> FontFamily {
    match fontname.to_lowercase().as_str() {
        "segoe ui" => FontFamily::Name("SegoeUI".into()),
        "segoe ui bold" => FontFamily::Name("SegoeUIBold".into()),
        "segoe ui italic" => FontFamily::Name("SegoeUIItalic".into()),
        "segoe ui bold italic" => FontFamily::Name("SegoeUIBoldItalic".into()),
        "sans-serif" => FontFamily::Name("SansSerif".into()),
        "sans-serif bold" => FontFamily::Name("SansSerifBold".into()),
        "sans-serif italic" => FontFamily::Name("SansSerifItalic".into()),
        "algerian" => FontFamily::Name("Algerian".into()),
        "arial" | "helvetica" | "proportional" => FontFamily::Proportional,
        "courier" | "monospace" => FontFamily::Monospace,
        _ => FontFamily::Proportional, // Fallback
    }
}

pub fn setdock(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("setdock() expects 2 arguments (control_id, dock_style), got {}", args.len())
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setdock() expects a control identifier".to_string());
        }
    };
    let dock_style = match &args[1] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err("setdock() expects a string for dock style".to_string());
        }
    };
    let dock = match dock_style.as_str() {
        "none" => DockStyle::None,
        "top" => DockStyle::Top,
        "bottom" => DockStyle::Bottom,
        "left" => DockStyle::Left,
        "right" => DockStyle::Right,
        "fill" => DockStyle::Fill,
        _ => {
            return Err(
                "Invalid dock style: use 'none', 'top', 'bottom', 'left', 'right', 'fill'".to_string()
            );
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.dock = dock;
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn getdock(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getdock() expects 1 argument (control_id), got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getdock() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        let dock_str = match control.dock {
            DockStyle::None => "none",
            DockStyle::Top => "top",
            DockStyle::Bottom => "bottom",
            DockStyle::Left => "left",
            DockStyle::Right => "right",
            DockStyle::Fill => "fill",
        };
        Ok(Value::String(dock_str.to_string()))
    } else {
        Err("Control not found".to_string())
    }
}

pub fn setclickhandler(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!(
                "setclickhandler() expects 2 arguments (control_id, callback), got {}",
                args.len()
            )
        );
    }

    // Extract control identifier (must be a string)
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setclickhandler() expects a control identifier".to_string());
        }
    };

    // Extract the callback value
    let callback = Some(args[1].clone());

    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        // Allow for both "button" and "imagebutton" control types.
        match control.control_type.as_str() {
            "button" | "imagebutton" => {
                control.callback = callback;
                Ok(Value::Null)
            }
            _ => Err("setclickhandler() can only be used with buttons or imagebuttons".to_string()),
        }
    } else {
        Err("Control not found".to_string())
    }
}

pub fn setenabled(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("setenabled() expects 2 arguments (control_id, enabled), got {}", args.len())
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setenabled() expects a control identifier".to_string());
        }
    };
    let enabled = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("setenabled() expects a boolean".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.enabled = enabled;
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn getenabled(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getenabled() expects 1 argument (control_id), got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getenabled() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        Ok(Value::Bool(control.enabled))
    } else {
        Err("Control not found".to_string())
    }
}

pub fn setvisibility(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("setvisibility() expects 2 arguments (control_id, visible), got {}", args.len())
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setvisibility() expects a control identifier".to_string());
        }
    };
    let visible = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("setvisibility() expects a boolean".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        control.visible = visible;
        Ok(Value::Null)
    } else {
        Err("Control not found".to_string())
    }
}

pub fn getvisibility(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getvisibility() expects 1 argument (control_id), got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getvisibility() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        Ok(Value::Bool(control.visible))
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_cursor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("set_cursor() expects 2 arguments (control_id, cursor), got {}", args.len())
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_cursor() expects a control identifier".to_string());
        }
    };
    let cursor = match &args[1] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err("set_cursor() expects a string for cursor".to_string());
        }
    };

    // Validate cursor value
    let valid_cursors = [
        "hand",
        "pointer",
        "default",
        "crosshair",
        "text",
        "move",
        "grab",
        "grabbing",
    ];
    if !valid_cursors.contains(&cursor.as_str()) {
        return Err(format!("Invalid cursor: use one of {:?}", valid_cursors));
    }

    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        if control.control_type == "button" {
            control.cursor = cursor;
            Ok(Value::Null)
        } else {
            Err("set_cursor() can only be used with buttons".to_string())
        }
    } else {
        Err("Control not found".to_string())
    }
}

pub fn get_cursor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_cursor() expects 1 argument (control_id), got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_cursor() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        if control.control_type == "button" {
            Ok(Value::String(control.cursor.clone()))
        } else {
            Err("get_cursor() can only be used with buttons".to_string())
        }
    } else {
        Err("Control not found".to_string())
    }
}

pub fn set_multiline(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!(
                "set_multiline() expects 2 arguments (control_id, multiline), got {}",
                args.len()
            )
        );
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_multiline() expects a control identifier".to_string());
        }
    };
    let multiline = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_multiline() expects a boolean".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&control_id) {
        if control.control_type == "textbox" {
            control.multiline = multiline;
            Ok(Value::Null)
        } else {
            Err("set_multiline() can only be used with textboxes".to_string())
        }
    } else {
        Err("Control not found".to_string())
    }
}

pub fn get_multiline(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_multiline() expects 1 argument (control_id), got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_multiline() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        if control.control_type == "textbox" {
            Ok(Value::Bool(control.multiline))
        } else {
            Err("get_multiline() can only be used with textboxes".to_string())
        }
    } else {
        Err("Control not found".to_string())
    }
}

pub fn setchecked(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setchecked() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setchecked() expects a control identifier".to_string());
        }
    };
    let checked = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("setchecked() expects a boolean".to_string());
        }
    };

    let mut controls = CONTROLS.write().unwrap(); // First mutable borrow starts

    // Step 1: Get info immutably first
    let (form_id, group) = if let Some(control) = controls.get(&control_id) {
        // Immutable borrow
        (control.form_id.clone(), control.group.clone())
    } else {
        return Err("Control not found".to_string());
    };

    // Step 2: Update the target control
    if let Some(control) = controls.get_mut(&control_id) {
        match control.control_type.as_str() {
            "checkbox" => {
                control.checked = checked;
            }
            "radiobox" => {
                if checked {
                    if let Some(group) = &group {
                        // Update radio group state
                        let mut selected_radioboxes = SELECTED_RADIOBOXES.write().unwrap();
                        selected_radioboxes.insert(
                            (form_id.clone(), group.clone()),
                            control_id.clone()
                        );
                        drop(selected_radioboxes);

                        // Step 3: Collect IDs to update (immutable iteration)
                        let ids_in_group: Vec<String> = controls
                            .iter()
                            .filter(
                                |(_id, ctrl)|
                                    ctrl.form_id == form_id && ctrl.group.as_ref() == Some(group)
                            )
                            .map(|(id, _)| id.clone())
                            .collect();

                        // Step 4: Update each control mutably
                        for id in ids_in_group {
                            if let Some(ctrl) = controls.get_mut(&id) {
                                ctrl.checked = id == control_id;
                            }
                        }
                    }
                }
            }
            _ => {
                return Err("setchecked() only works with checkboxes and radioboxes".to_string());
            }
        }
    }

    Ok(Value::Null)
}

pub fn getchecked(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getchecked() expects 1 argument (control_id), got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getchecked() expects a control identifier".to_string());
        }
    };
    let controls = CONTROLS.read().unwrap();
    if let Some(control) = controls.get(&control_id) {
        match control.control_type.as_str() {
            "checkbox" | "radiobox" => Ok(Value::Bool(control.checked)),
            _ => Err("getchecked() can only be used with checkboxes and radioboxes".to_string()),
        }
    } else {
        Err("Control not found".to_string())
    }
}

// A generic function to get the selected index for both ListBox and ComboBox.
pub fn get_selected_index(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_selected_index() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_selected_index() expects a control identifier".to_string());
        }
    };

    // Read the control to determine its type.
    let controls = CONTROLS.read().unwrap();
    let control = controls.get(&control_id).ok_or("Control not found".to_string())?;
    match control.control_type.as_str() {
        "combobox" => {
            let combobox_items = COMBOBOX_ITEMS.read().unwrap();
            if let Some(items_map) = combobox_items.get(&control_id) {
                if let Some((_, selected_index)) = items_map.get(&control_id) {
                    Ok(Value::Number(*selected_index as f64))
                } else {
                    Err("ComboBox not found".to_string())
                }
            } else {
                Err("ComboBox not found".to_string())
            }
        }
        "listbox" => {
            let listbox_items = LISTBOX_ITEMS.read().unwrap();
            if let Some(items_map) = listbox_items.get(&control_id) {
                if let Some((_, selected_index)) = items_map.get(&control_id) {
                    Ok(Value::Number(*selected_index as f64))
                } else {
                    Err("ListBox not found".to_string())
                }
            } else {
                Err("ListBox not found".to_string())
            }
        }
        _ => Err("Unsupported control type for get_selected_index".to_string()),
    }
}
pub fn set_selected_text(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_selected_text() expects 2 arguments, got {}", args.len()));
    }

    // Extract control identifier.
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err(
                "set_selected_text() expects a control identifier as the first argument".to_string()
            );
        }
    };

    // Extract the new text value.
    let new_text = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_selected_text() expects a string for the new text".to_string());
        }
    };

    // Use an inner block to determine the control type.
    let control_type = {
        let controls = CONTROLS.read().unwrap();
        let control = controls.get(&control_id).ok_or("Control not found".to_string())?;
        control.control_type.clone()
    }; // controls lock is dropped here

    // Proceed based on the control type.
    match control_type.as_str() {
        "combobox" => {
            let mut items_lock = COMBOBOX_ITEMS.write().unwrap();
            if let Some(items_map) = items_lock.get_mut(&control_id) {
                if let Some((items, selected_index)) = items_map.get_mut(&control_id) {
                    // Try to find the new text in the items list.
                    if
                        let Some((idx, _)) = items
                            .iter()
                            .enumerate()
                            .find(|(_, item)| {
                                let item_value = item.lock().unwrap();
                                item_value.to_string() == new_text
                            })
                    {
                        *selected_index = idx;
                        // Update the control's text.
                        let mut controls = CONTROLS.write().unwrap();
                        if let Some(ctrl) = controls.get_mut(&control_id) {
                            ctrl.text = new_text.clone();
                        }
                        Ok(Value::Null)
                    } else {
                        Err("Text not found in ComboBox items".to_string())
                    }
                } else {
                    Err("ComboBox not found".to_string())
                }
            } else {
                Err("ComboBox not found".to_string())
            }
        }
        "listbox" => {
            let mut items_lock = LISTBOX_ITEMS.write().unwrap();
            if let Some(items_map) = items_lock.get_mut(&control_id) {
                if let Some((items, selected_index)) = items_map.get_mut(&control_id) {
                    // Try to find the new text in the items list.
                    if
                        let Some((idx, _)) = items
                            .iter()
                            .enumerate()
                            .find(|(_, item)| {
                                let item_value = item.lock().unwrap();
                                item_value.to_string() == new_text
                            })
                    {
                        *selected_index = idx;
                        // Update the control's text.
                        let mut controls = CONTROLS.write().unwrap();
                        if let Some(ctrl) = controls.get_mut(&control_id) {
                            ctrl.text = new_text.clone();
                        }
                        Ok(Value::Null)
                    } else {
                        Err("Text not found in ListBox items".to_string())
                    }
                } else {
                    Err("ListBox not found".to_string())
                }
            } else {
                Err("ListBox not found".to_string())
            }
        }
        _ => Err("Unsupported control type for set_selected_text".to_string()),
    }
}

// A generic function to set the selected index for both ListBox and ComboBox.
pub fn set_selected_index(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_selected_index() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_selected_index() expects a control identifier".to_string());
        }
    };
    let index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("set_selected_index() expects a number".to_string());
        }
    };

    // Use an inner block to get the control type.
    let control_type = {
        let controls = CONTROLS.read().unwrap();
        let control = controls.get(&control_id).ok_or("Control not found".to_string())?;
        control.control_type.clone()
    }; // The read lock on CONTROLS is dropped here.

    match control_type.as_str() {
        "combobox" => {
            let mut combobox_items = COMBOBOX_ITEMS.write().unwrap();
            if let Some(items_map) = combobox_items.get_mut(&control_id) {
                if let Some((items, selected_index)) = items_map.get_mut(&control_id) {
                    if index < items.len() {
                        *selected_index = index;
                        // Update the control text.
                        let mut controls = CONTROLS.write().unwrap();
                        if let Some(ctrl) = controls.get_mut(&control_id) {
                            let value = items[index].lock().unwrap(); // Access the inner Value
                            ctrl.text = value.to_string();
                        }
                        Ok(Value::Null)
                    } else {
                        Err("Index out of bounds".to_string())
                    }
                } else {
                    Err("ComboBox not found".to_string())
                }
            } else {
                Err("ComboBox not found".to_string())
            }
        }
        "listbox" => {
            let mut listbox_items = LISTBOX_ITEMS.write().unwrap();
            if let Some(items_map) = listbox_items.get_mut(&control_id) {
                if let Some((items, selected_index)) = items_map.get_mut(&control_id) {
                    if index < items.len() {
                        *selected_index = index;
                        // Update the control text.
                        let mut controls = CONTROLS.write().unwrap();
                        if let Some(ctrl) = controls.get_mut(&control_id) {
                            let value = items[index].lock().unwrap();
                            ctrl.text = value.to_string();
                        }
                        Ok(Value::Null)
                    } else {
                        Err("Index out of bounds".to_string())
                    }
                } else {
                    Err("ListBox not found".to_string())
                }
            } else {
                Err("ListBox not found".to_string())
            }
        }
        _ => Err("Unsupported control type for set_selected_index".to_string()),
    }
}

// A generic function to get the selected text from either a ListBox or ComboBox.
pub fn get_selected_text(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_selected_text() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_selected_text() expects a control identifier".to_string());
        }
    };

    let controls = CONTROLS.read().unwrap();
    let control = controls.get(&control_id).ok_or("Control not found".to_string())?;
    match control.control_type.as_str() {
        "combobox" | "listbox" => Ok(Value::String(control.text.clone())),
        _ => Err("Unsupported control type for get_selected_text".to_string()),
    }
}

pub fn additem(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err(
            format!(
                "additem() expects 2-3 arguments (control_id, item(s), [index]), got {}",
                args.len()
            )
        );
    }

    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("additem() expects a control identifier".to_string());
        }
    };

    let items_to_add = match &args[1] {
        Value::String(s) => vec![Arc::new(Mutex::new(Value::String(s.clone())))],
        Value::Array(arr) => arr.clone(),
        _ => {
            return Err("additem() expects a string or array for item(s)".to_string());
        }
    };

    let index = args.get(2).map(|v| {
        match v {
            Value::Number(n) => *n as usize,
            _ => 0,
        }
    });

    let controls = CONTROLS.read().unwrap();
    let control = controls.get(&control_id).ok_or("Control not found".to_string())?;

    match control.control_type.as_str() {
        "listbox" => {
            let mut listbox_items = LISTBOX_ITEMS.write().unwrap();
            if let Some(items_map) = listbox_items.get_mut(&control_id) {
                if let Some((items, selected_index)) = items_map.get_mut(&control_id) {
                    match index {
                        Some(idx) if idx <= items.len() => {
                            items.splice(idx..idx, items_to_add.into_iter());
                        }
                        None => items.extend(items_to_add),
                        _ => {
                            return Err("Index out of bounds".to_string());
                        }
                    }
                    Ok(Value::Null)
                } else {
                    Err("ListBox not found".to_string())
                }
            } else {
                Err("ListBox not found".to_string())
            }
        }
        "combobox" => {
            let mut combobox_items = COMBOBOX_ITEMS.write().unwrap();
            if let Some(items_map) = combobox_items.get_mut(&control_id) {
                if let Some((items, selected_index)) = items_map.get_mut(&control_id) {
                    match index {
                        Some(idx) if idx <= items.len() => {
                            items.splice(idx..idx, items_to_add.into_iter());
                        }
                        None => items.extend(items_to_add),
                        _ => {
                            return Err("Index out of bounds".to_string());
                        }
                    }
                    Ok(Value::Null)
                } else {
                    Err("ComboBox not found".to_string())
                }
            } else {
                Err("ComboBox not found".to_string())
            }
        }
        _ => Err("additem() only works with ListBox or ComboBox".to_string()),
    }
}

pub fn getitem(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("getitem() expects 2 arguments (control_id, index), got {}", args.len())
        );
    }

    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getitem() expects a control identifier".to_string());
        }
    };

    let index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("getitem() expects a number for index".to_string());
        }
    };

    let controls = CONTROLS.read().unwrap();
    let control = controls.get(&control_id).ok_or("Control not found".to_string())?;

    match control.control_type.as_str() {
        "listbox" => {
            let listbox_items = LISTBOX_ITEMS.read().unwrap();
            if let Some(items_map) = listbox_items.get(&control_id) {
                if let Some((items, _)) = items_map.get(&control_id) {
                    if index < items.len() {
                        let item = items[index].lock().unwrap();
                        Ok(item.clone())
                    } else {
                        Err("Index out of bounds".to_string())
                    }
                } else {
                    Err("ListBox not found".to_string())
                }
            } else {
                Err("ListBox not found".to_string())
            }
        }
        "combobox" => {
            let combobox_items = COMBOBOX_ITEMS.read().unwrap();
            if let Some(items_map) = combobox_items.get(&control_id) {
                if let Some((items, _)) = items_map.get(&control_id) {
                    if index < items.len() {
                        let item = items[index].lock().unwrap();
                        Ok(item.clone())
                    } else {
                        Err("Index out of bounds".to_string())
                    }
                } else {
                    Err("ComboBox not found".to_string())
                }
            } else {
                Err("ComboBox not found".to_string())
            }
        }
        _ => Err("getitem() only works with ListBox or ComboBox".to_string()),
    }
}

pub fn setscrollvalue(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setscrollvalue() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setscrollvalue() expects a control identifier".to_string());
        }
    };
    let value = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("setscrollvalue() expects a number".to_string());
        }
    };
    let mut states = SCROLLBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.value = value.clamp(state.min, state.max);
        Ok(Value::Null)
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn getscrollvalue(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getscrollvalue() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getscrollvalue() expects a control identifier".to_string());
        }
    };
    let states = SCROLLBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        Ok(Value::Number(state.value as f64))
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn setscrollmin(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setscrollmin() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setscrollmin() expects a control identifier".to_string());
        }
    };
    let min = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("setscrollmin() expects a number".to_string());
        }
    };
    let mut states = SCROLLBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.min = min;
        state.value = state.value.max(min);
        Ok(Value::Null)
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn getscrollmin(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getscrollmin() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getscrollmin() expects a control identifier".to_string());
        }
    };
    let states = SCROLLBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        Ok(Value::Number(state.min as f64))
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn setscrollmax(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setscrollmax() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setscrollmax() expects a control identifier".to_string());
        }
    };
    let max = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("setscrollmax() expects a number".to_string());
        }
    };
    let mut states = SCROLLBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.max = max;
        state.value = state.value.min(max);
        Ok(Value::Null)
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn getscrollmax(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getscrollmax() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getscrollmax() expects a control identifier".to_string());
        }
    };
    let states = SCROLLBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        Ok(Value::Number(state.max as f64))
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn setlargechange(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setlargechange() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setlargechange() expects a control identifier".to_string());
        }
    };
    let large_change = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("setlargechange() expects a number".to_string());
        }
    };
    let mut states = SCROLLBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.large_change = large_change.max(0.0);
        Ok(Value::Null)
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn getlargechange(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getlargechange() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getlargechange() expects a control identifier".to_string());
        }
    };
    let states = SCROLLBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        Ok(Value::Number(state.large_change as f64))
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn setsmallchange(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setsmallchange() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setsmallchange() expects a control identifier".to_string());
        }
    };
    let small_change = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("setsmallchange() expects a number".to_string());
        }
    };
    let mut states = SCROLLBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.small_change = small_change.max(0.0);
        Ok(Value::Null)
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn getsmallchange(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getsmallchange() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getsmallchange() expects a control identifier".to_string());
        }
    };
    let states = SCROLLBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        Ok(Value::Number(state.small_change as f64))
    } else {
        Err("ScrollBar not found".to_string())
    }
}

pub fn setimage(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setimage() expects 2 arguments, got {}", args.len()));
    }

    // Extract the control identifier (expected as a string)
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setimage() expects a control identifier (string)".to_string());
        }
    };

    // Extract the image path (expected as a string)
    let path = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("setimage() expects a string for the image path".to_string());
        }
    };

    // Try updating for an ImageButton first
    {
        let mut imagebutton_states = IMAGEBUTTON_STATES.write().unwrap();
        if let Some(state) = imagebutton_states.get_mut(&control_id) {
            state.image_path = Some(path.clone());
            state.texture_handle = None; // Reset texture to force a reload
            return Ok(Value::Null);
        }
    }

    // Try updating for a PictureBox
    {
        let mut picturebox_states = PICTUREBOX_STATES.write().unwrap();
        if let Some(state) = picturebox_states.get_mut(&control_id) {
            state.image_path = Some(path);
            state.texture_handle = None; // Reset texture to force a reload
            return Ok(Value::Null);
        }
    }

    Err("Control not found. Supported controls: ImageButton, PictureBox".to_string())
}

pub fn setsizemode(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setsizemode() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setsizemode() expects a control identifier".to_string());
        }
    };
    let mode = match &args[1] {
        Value::String(s) =>
            match s.to_lowercase().as_str() {
                "normal" => PictureBoxSizeMode::Normal,
                "stretch" => PictureBoxSizeMode::Stretch,
                "zoom" => PictureBoxSizeMode::Zoom,
                _ => {
                    return Err("setsizemode() expects 'normal', 'stretch', or 'zoom'".to_string());
                }
            }
        _ => {
            return Err("setsizemode() expects a string".to_string());
        }
    };
    let mut states = PICTUREBOX_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.size_mode = mode;
        Ok(Value::Null)
    } else {
        Err("PictureBox not found".to_string())
    }
}

pub fn setbarcolor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setbarcolor() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setbarcolor() expects a control identifier".to_string());
        }
    };
    let color = match &args[1] {
        Value::String(s) => parse_color(s).unwrap_or(Color32::BLUE),
        _ => {
            return Err("setbarcolor() expects a color string".to_string());
        }
    };
    let mut states = PROGRESSBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.bar_color = color;
        Ok(Value::Null)
    } else {
        Err("ProgressBar not found".to_string())
    }
}

pub fn getbarcolor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getbarcolor() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getbarcolor() expects a control identifier".to_string());
        }
    };
    let states = PROGRESSBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        let c = state.bar_color;
        Ok(Value::String(format!("{},{},{}", c.r(), c.g(), c.b())))
    } else {
        Err("ProgressBar not found".to_string())
    }
}

pub fn setvalue(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setvalue() expects 2 arguments, got {}", args.len()));
    }

    // Extract the control ID and the value
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setvalue() expects a control identifier".to_string());
        }
    };

    let value = match &args[1] {
        Value::Number(n) => *n, // Keep as f64 (for number box case)
        _ => {
            return Err("setvalue() expects a number".to_string());
        }
    };

    // Check for ProgressBar
    let mut states = PROGRESSBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        let value_f32 = value as f32; // Convert value to f32 for ProgressBar
        state.value = value_f32.clamp(state.min as f32, state.max as f32); // clamp for f32
        return Ok(Value::Null);
    }

    // Check for Slider
    let mut states = SLIDER_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        let value_f32 = value as f32; // Convert value to f32 for Slider
        state.value = value_f32.clamp(state.min as f32, state.max as f32); // clamp for f32
        return Ok(Value::Null);
    }

    // Check for NumberBox (which uses f64)
    let mut states = NUMBERBOX_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        let value_f64 = value; // Keep as f64 for NumberBox
        state.value = value_f64.clamp(state.min, state.max); // clamp for f64
        let mut controls = CONTROLS.write().unwrap();
        if let Some(ctrl) = controls.get_mut(&control_id) {
            ctrl.text = format!("{:.1$}", state.value, state.decimals);
        }
        return Ok(Value::Null);
    }

    // If no control found, return an error with supported controls
    Err(format!("Control not found. Supported controls: ProgressBar, Slider, NumberBox"))
}

pub fn getvalue(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getvalue() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getvalue() expects a control identifier".to_string());
        }
    };

    // Check for ProgressBar
    let states = PROGRESSBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.value as f64));
    }

    // Check for Slider
    let states = SLIDER_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.value as f64));
    }

    // Check for NumberBox
    let states = NUMBERBOX_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.value));
    }

    // If no control found, return an error with supported controls
    Err(format!("Control not found. Supported controls: ProgressBar, Slider, NumberBox"))
}

pub fn setmin(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setmin() expects 2 arguments, got {}", args.len()));
    }

    // Extract the control ID and the min value
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setmin() expects a control identifier".to_string());
        }
    };
    let min = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("setmin() expects a number".to_string());
        }
    };

    // Check for ProgressBar
    let mut states = PROGRESSBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.min = min;
        state.value = state.value.max(min);
        return Ok(Value::Null);
    }

    // Check for Slider
    let mut states = SLIDER_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.min = min;
        state.value = state.value.max(min);
        return Ok(Value::Null);
    }

    // Check for NumberBox
    let mut states = NUMBERBOX_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.min = min as f64;
        state.value = state.value.max(min as f64);
        let mut controls = CONTROLS.write().unwrap();
        if let Some(ctrl) = controls.get_mut(&control_id) {
            ctrl.text = format!("{:.1$}", state.value, state.decimals);
        }
        return Ok(Value::Null);
    }

    // If no control found, return an error with supported controls
    Err(format!("Control not found. Supported controls: ProgressBar, Slider, NumberBox"))
}

pub fn getmin(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getmin() expects 1 argument, got {}", args.len()));
    }

    // Extract the control ID from the first argument
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getmin() expects a control identifier".to_string());
        }
    };

    // Check for ProgressBar
    let states = PROGRESSBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.min as f64));
    }

    // Check for Slider
    let states = SLIDER_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.min as f64));
    }

    // Check for NumberBox
    let states = NUMBERBOX_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.min));
    }

    Err(format!("Control not found. Supported controls: ProgressBar, Slider, NumberBox"))
}

pub fn setmax(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setmax() expects 2 arguments, got {}", args.len()));
    }

    // Extract the control ID from the first argument
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setmax() expects a control identifier".to_string());
        }
    };

    // Extract the max value from the second argument
    let max = match &args[1] {
        Value::Number(n) => *n,
        _ => {
            return Err("setmax() expects a number".to_string());
        }
    };

    // Check if the control is a ProgressBar
    let mut progressbar_states = PROGRESSBAR_STATES.write().unwrap();
    if let Some(state) = progressbar_states.get_mut(&control_id) {
        state.max = max as f32;
        state.value = state.value.min(max as f32); // Ensure value doesn't exceed max
        return Ok(Value::Null);
    }

    // Check if the control is a Slider
    let mut slider_states = SLIDER_STATES.write().unwrap();
    if let Some(state) = slider_states.get_mut(&control_id) {
        state.max = max as f32;
        state.value = state.value.min(max as f32); // Ensure value doesn't exceed max
        return Ok(Value::Null);
    }

    // Check if the control is a NumberBox
    let mut numberbox_states = NUMBERBOX_STATES.write().unwrap();
    if let Some(state) = numberbox_states.get_mut(&control_id) {
        state.max = max;
        state.value = state.value.min(max); // Ensure value doesn't exceed max
        let mut controls = CONTROLS.write().unwrap();
        if let Some(ctrl) = controls.get_mut(&control_id) {
            ctrl.text = format!("{:.1$}", state.value, state.decimals);
        }
        return Ok(Value::Null);
    }

    // If no control found, return an error
    Err("Control not found".to_string())
}

pub fn getmax(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getmax() expects 1 argument, got {}", args.len()));
    }

    // Extract the control ID from the first argument
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getmax() expects a control identifier".to_string());
        }
    };

    // Check for ProgressBar
    let states = PROGRESSBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.max as f64));
    }

    // Check for Slider
    let states = SLIDER_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.max as f64));
    }

    // Check for NumberBox
    let states = NUMBERBOX_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        return Ok(Value::Number(state.max));
    }

    // If no control found, return an error
    Err("Control (Progrssbar, Slider, Numberbox) not found".to_string())
}

pub fn setbarstyle(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("setbarstyle() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("setbarstyle() expects a control identifier".to_string());
        }
    };
    let style = match &args[1] {
        Value::String(s) =>
            match s.to_lowercase().as_str() {
                "solid" => ProgressBarStyle::Solid,
                "marquee" => ProgressBarStyle::Marquee,
                _ => {
                    return Err("setbarstyle() expects 'solid' or 'marquee'".to_string());
                }
            }
        _ => {
            return Err("setbarstyle() expects a string".to_string());
        }
    };
    let mut states = PROGRESSBAR_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        state.style = style;
        Ok(Value::Null)
    } else {
        Err("ProgressBar not found".to_string())
    }
}

pub fn getbarstyle(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getbarstyle() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getbarstyle() expects a control identifier".to_string());
        }
    };
    let states = PROGRESSBAR_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        let style_str = match state.style {
            ProgressBarStyle::Solid => "solid",
            ProgressBarStyle::Marquee => "marquee",
        };
        Ok(Value::String(style_str.to_string()))
    } else {
        Err("ProgressBar not found".to_string())
    }
}

fn ui_text_size(ui: &mut egui::Ui, text: &str, control: &ControlSettings) -> (f32, f32) {
    // Use the mapped font family
    let font_id = FontId::new(control.fontsize, get_font_family(&control.fontname));
    let mut job = egui::text::LayoutJob::simple_singleline(
        text.to_string(),
        font_id,
        control.forecolor
    );
    job.wrap.max_width = f32::INFINITY;
    let galley = ui.fonts(|f| f.layout_job(job));
    (galley.rect.width(), galley.rect.height())
}

pub fn showpage(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("showpage() expects 2 arguments, got {}", args.len()));
    }
    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a Pages ID (string)".to_string());
        }
    };
    let page_index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("Second argument must be a page index (number)".to_string());
        }
    };

    let mut pages_states = PAGES_STATES.write().unwrap();
    if let Some(state) = pages_states.get_mut(&pages_id) {
        if page_index < state.pages.len() {
            if state.active_page_index != page_index {
                // Only trigger if switching
                state.active_page_index = page_index;
                if state.use_transition {
                    state.in_transition = true;
                    state.transition_alpha = 0.0;
                    state.transition_start_time = Some(std::time::Instant::now());
                }
            }
            Ok(Value::Null)
        } else {
            Err("Page index out of bounds".to_string())
        }
    } else {
        Err("Pages ID not found".to_string())
    }
}

pub fn hidepage(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("hidepage() expects 2 arguments, got {}", args.len()));
    }
    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a Pages ID (string)".to_string());
        }
    };
    let page_index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("Second argument must be a page index (number)".to_string());
        }
    };

    let mut pages_states = PAGES_STATES.write().unwrap();
    if let Some(state) = pages_states.get_mut(&pages_id) {
        if state.active_page_index == page_index && state.pages.len() > 1 {
            // Switch to the next page (or first page if at the end)
            state.active_page_index = (page_index + 1) % state.pages.len();
            state.in_transition = true;
            state.transition_alpha = 0.0;
            state.transition_start_time = Some(std::time::Instant::now());
        }
        Ok(Value::Null)
    } else {
        Err("Pages ID not found".to_string())
    }
}

pub fn setpageindex(args: Vec<Value>) -> Result<Value, String> {
    showpage(args) // Reuse showpage implementation
}

pub fn getpageindex(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getindex() expects 1 argument, got {}", args.len()));
    }
    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a Pages ID (string)".to_string());
        }
    };

    let pages_states = PAGES_STATES.read().unwrap();
    if let Some(state) = pages_states.get(&pages_id) {
        Ok(Value::Number(state.active_page_index as f64))
    } else {
        Err("Pages ID not found".to_string())
    }
}

pub fn addpage(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("addpage() expects 2 arguments, got {}", args.len()));
    }
    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a Pages ID (string)".to_string());
        }
    };
    let title = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("Second argument must be a title (string)".to_string());
        }
    };

    let mut pages_states = PAGES_STATES.write().unwrap();
    if let Some(state) = pages_states.get_mut(&pages_id) {
        state.pages.push(Page { title, control_ids: Vec::new() });
        Ok(Value::Null)
    } else {
        Err("Pages ID not found".to_string())
    }
}

pub fn removepage(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("removepage() expects 2 arguments, got {}", args.len()));
    }
    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a Pages ID (string)".to_string());
        }
    };
    let page_index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("Second argument must be a page index (number)".to_string());
        }
    };

    let mut pages_states = PAGES_STATES.write().unwrap();
    if let Some(state) = pages_states.get_mut(&pages_id) {
        if page_index < state.pages.len() {
            state.pages.remove(page_index);
            if state.active_page_index >= state.pages.len() {
                state.active_page_index = state.pages.len().saturating_sub(1);
            }
            Ok(Value::Null)
        } else {
            Err("Page index out of bounds".to_string())
        }
    } else {
        Err("Pages ID not found".to_string())
    }
}

pub fn settransition(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("settransition() expects 2 arguments (pages_id, enable), got {}", args.len())
        );
    }

    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a Pages ID (string)".to_string());
        }
    };

    let enable = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("Second argument must be a boolean".to_string());
        }
    };

    let mut pages_states = PAGES_STATES.write().unwrap();
    if let Some(state) = pages_states.get_mut(&pages_id) {
        state.use_transition = enable;
        if !enable {
            // If disabling transitions, reset animation state
            state.in_transition = false;
            state.transition_alpha = 1.0;
            state.transition_start_time = None;
        }
        Ok(Value::Null)
    } else {
        Err("Pages ID not found".to_string())
    }
}

pub fn set_use_as_default_panel(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_use_as_default_panel() expects 2 arguments, got {}", args.len()));
    }
    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_use_as_default_panel() expects a Pages identifier".to_string());
        }
    };
    let enable = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_use_as_default_panel() expects a boolean".to_string());
        }
    };
    let mut controls = CONTROLS.write().unwrap();
    if let Some(control) = controls.get_mut(&pages_id) {
        if control.control_type == "pages" {
            control.use_as_default_panel = enable;
            Ok(Value::Null)
        } else {
            Err("Control must be a Pages type".to_string())
        }
    } else {
        Err("Pages control not found".to_string())
    }
}

pub fn settimerenabled(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("settimerenabled() expects 2 arguments (timer_id, enabled), got {}", args.len())
        );
    }
    let timer_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("settimerenabled() expects a timer identifier".to_string());
        }
    };
    let enabled = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("settimerenabled() expects a boolean".to_string());
        }
    };
    let mut timers = TIMER_STATES.write().unwrap();
    if let Some(timer) = timers.get_mut(&timer_id) {
        timer.enabled = enabled;
        if enabled && timer.last_tick.is_none() {
            timer.last_tick = Some(Instant::now()); // Initialize on first enable
        }
        Ok(Value::Null)
    } else {
        Err("Timer not found".to_string())
    }
}

pub fn gettimerenabled(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("gettimerenabled() expects 1 argument (timer_id), got {}", args.len()));
    }
    let timer_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("gettimerenabled() expects a timer identifier".to_string());
        }
    };
    let timers = TIMER_STATES.read().unwrap();
    if let Some(timer) = timers.get(&timer_id) {
        Ok(Value::Bool(timer.enabled))
    } else {
        Err("Timer not found".to_string())
    }
}

pub fn settimerinterval(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!(
                "settimerinterval() expects 2 arguments (timer_id, interval), got {}",
                args.len()
            )
        );
    }
    let timer_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("settimerinterval() expects a timer identifier".to_string());
        }
    };
    let interval = match &args[1] {
        Value::Number(n) => *n as u32,
        _ => {
            return Err("settimerinterval() expects a number".to_string());
        }
    };
    let mut timers = TIMER_STATES.write().unwrap();
    if let Some(timer) = timers.get_mut(&timer_id) {
        timer.interval = interval;
        Ok(Value::Null)
    } else {
        Err("Timer not found".to_string())
    }
}

pub fn gettimerinterval(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("gettimerinterval() expects 1 argument (timer_id), got {}", args.len()));
    }
    let timer_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("gettimerinterval() expects a timer identifier".to_string());
        }
    };
    let timers = TIMER_STATES.read().unwrap();
    if let Some(timer) = timers.get(&timer_id) {
        Ok(Value::Number(timer.interval as f64))
    } else {
        Err("Timer not found".to_string())
    }
}
pub fn starttimer(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("starttimer() expects 1 argument (timer_id), got {}", args.len()));
    }
    let timer_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("starttimer() expects a timer identifier".to_string());
        }
    };
    let mut timers = TIMER_STATES.write().unwrap();
    if let Some(timer) = timers.get_mut(&timer_id) {
        timer.enabled = true;
        if timer.last_tick.is_none() {
            timer.last_tick = Some(Instant::now());
        }
        Ok(Value::Null)
    } else {
        Err("Timer not found".to_string())
    }
}

pub fn stoptimer(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("stoptimer() expects 1 argument (timer_id), got {}", args.len()));
    }
    let timer_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("stoptimer() expects a timer identifier".to_string());
        }
    };
    let mut timers = TIMER_STATES.write().unwrap();
    if let Some(timer) = timers.get_mut(&timer_id) {
        timer.enabled = false;
        Ok(Value::Null)
    } else {
        Err("Timer not found".to_string())
    }
}

pub fn save_file_dialog(args: Vec<Value>) -> Result<Value, String> {
    if args.len() > 4 {
        return Err(
            format!(
                "save_file_dialog() expects 0-4 arguments (title, multiselect, startingpath, filters), got {}",
                args.len()
            )
        );
    }

    // Clone global options and update with arguments
    let options = {
        let mut opts = FILE_DIALOG_OPTIONS.lock().unwrap().clone();
        if let Some(Value::String(title)) = args.get(0) {
            opts.title = Some(title.clone());
        }
        // Note: multiselect is ignored, as save_file() only supports single selection
        if let Some(Value::Bool(_multiselect)) = args.get(1) {
            // Optionally, return an error if multiselect is true:
            // return Err("save_file_dialog does not support multiselect".to_string());
        }
        if let Some(Value::String(path)) = args.get(2) {
            opts.startingpath = Some(path.clone());
        }
        if let Some(Value::Array(filter_arr)) = args.get(3) {
            let mut filters = Vec::new();
            for filter in filter_arr.iter() {
                if let Value::Array(pair) = &*filter.lock().unwrap() {
                    if pair.len() == 2 {
                        if
                            let (Value::String(name), Value::Array(exts)) = (
                                &*pair[0].lock().unwrap(),
                                &*pair[1].lock().unwrap(),
                            )
                        {
                            let extensions: Vec<String> = exts
                                .iter()
                                .map(|e| e.lock().unwrap().to_string())
                                .collect();
                            filters.push((name.clone(), extensions));
                        }
                    }
                }
            }
            opts.filters = Some(filters);
        }
        opts
    };

    // Configure the dialog
    let mut dialog = RfdFileDialog::new();
    if let Some(title) = &options.title {
        dialog = dialog.set_title(title);
    }
    if let Some(path) = &options.startingpath {
        dialog = dialog.set_directory(path);
    }
    if let Some(filters) = &options.filters {
        for (name, exts) in filters {
            dialog = dialog.add_filter(name, exts);
        }
    }

    // Execute the dialog (always single selection)
    let result = dialog.save_file().map(|path| Value::String(path.to_string_lossy().to_string()));

    result.ok_or_else(|| "No file selected".to_string())
}

pub fn open_file_dialog(args: Vec<Value>) -> Result<Value, String> {
    if args.len() > 4 {
        return Err(
            format!(
                "open_file_dialog() expects 0-4 arguments (title, multiselect, startingpath, filters), got {}",
                args.len()
            )
        );
    }

    // Clone global options and update with arguments
    let options = {
        let mut opts = FILE_DIALOG_OPTIONS.lock().unwrap().clone();
        if let Some(Value::String(title)) = args.get(0) {
            opts.title = Some(title.clone());
        }
        if let Some(Value::Bool(multiselect)) = args.get(1) {
            opts.multiselect = *multiselect;
        }
        if let Some(Value::String(path)) = args.get(2) {
            opts.startingpath = Some(path.clone());
        }
        if let Some(Value::Array(filter_arr)) = args.get(3) {
            let mut filters = Vec::new();
            for filter in filter_arr.iter() {
                if let Value::Array(pair) = &*filter.lock().unwrap() {
                    if pair.len() == 2 {
                        if
                            let (Value::String(name), Value::Array(exts)) = (
                                &*pair[0].lock().unwrap(),
                                &*pair[1].lock().unwrap(),
                            )
                        {
                            let extensions: Vec<String> = exts
                                .iter()
                                .map(|e| e.lock().unwrap().to_string())
                                .collect();
                            filters.push((name.clone(), extensions));
                        }
                    }
                }
            }
            opts.filters = Some(filters);
        }
        opts
    };

    // Configure the dialog
    let mut dialog = RfdFileDialog::new();
    if let Some(title) = &options.title {
        dialog = dialog.set_title(title);
    }
    if let Some(path) = &options.startingpath {
        dialog = dialog.set_directory(path);
    }
    if let Some(filters) = &options.filters {
        for (name, exts) in filters {
            dialog = dialog.add_filter(name, exts);
        }
    }

    // Execute the dialog based on multiselect
    let result = if options.multiselect {
        dialog.pick_files().map(|paths| {
            Value::Array(
                paths
                    .into_iter()
                    .map(|path|
                        Arc::new(Mutex::new(Value::String(path.to_string_lossy().to_string())))
                    )
                    .collect()
            )
        })
    } else {
        dialog.pick_file().map(|path| Value::String(path.to_string_lossy().to_string()))
    };

    result.ok_or_else(|| "No file(s) selected".to_string())
}

pub fn folder_dialog(args: Vec<Value>) -> Result<Value, String> {
    if args.len() > 4 {
        return Err(
            format!(
                "folder_dialog() expects 0-4 arguments (title, multiselect, startingpath, filters), got {}",
                args.len()
            )
        );
    }

    // Clone global options and update with arguments
    let options = {
        let mut opts = FILE_DIALOG_OPTIONS.lock().unwrap().clone();
        if let Some(Value::String(title)) = args.get(0) {
            opts.title = Some(title.clone());
        }
        if let Some(Value::Bool(multiselect)) = args.get(1) {
            opts.multiselect = *multiselect;
        }
        if let Some(Value::String(path)) = args.get(2) {
            opts.startingpath = Some(path.clone());
        }
        // Filters are parsed but ignored by rfd for folder dialogs
        if let Some(Value::Array(filter_arr)) = args.get(3) {
            let mut filters = Vec::new();
            for filter in filter_arr.iter() {
                if let Value::Array(pair) = &*filter.lock().unwrap() {
                    if pair.len() == 2 {
                        if
                            let (Value::String(name), Value::Array(exts)) = (
                                &*pair[0].lock().unwrap(),
                                &*pair[1].lock().unwrap(),
                            )
                        {
                            let extensions: Vec<String> = exts
                                .iter()
                                .map(|e| e.lock().unwrap().to_string())
                                .collect();
                            filters.push((name.clone(), extensions));
                        }
                    }
                }
            }
            opts.filters = Some(filters);
        }
        opts
    };

    // Configure the dialog
    let mut dialog = RfdFileDialog::new();
    if let Some(title) = &options.title {
        dialog = dialog.set_title(title);
    }
    if let Some(path) = &options.startingpath {
        dialog = dialog.set_directory(path);
    }
    // Note: Filters are ignored by rfd for folder selection, so we skip adding them

    // Execute the dialog based on multiselect
    let result = if options.multiselect {
        dialog.pick_folders().map(|paths| {
            Value::Array(
                paths
                    .into_iter()
                    .map(|path|
                        Arc::new(Mutex::new(Value::String(path.to_string_lossy().to_string())))
                    )
                    .collect()
            )
        })
    } else {
        dialog.pick_folder().map(|path| Value::String(path.to_string_lossy().to_string()))
    };

    result.ok_or_else(|| "No folder(s) selected".to_string())
}

pub fn set_dialog_title(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("set_dialog_title() expects 1 argument (title), got {}", args.len()));
    }
    let title = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_dialog_title() expects a string".to_string());
        }
    };
    let mut options = FILE_DIALOG_OPTIONS.lock().unwrap();
    options.title = Some(title);
    Ok(Value::Null)
}

pub fn set_dialog_multiselect(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!("set_dialog_multiselect() expects 1 argument (multiselect), got {}", args.len())
        );
    }
    let multiselect = match &args[0] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_dialog_multiselect() expects a boolean".to_string());
        }
    };
    let mut options = FILE_DIALOG_OPTIONS.lock().unwrap();
    options.multiselect = multiselect;
    Ok(Value::Null)
}

pub fn set_dialog_startingpath(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!("set_dialog_startingpath() expects 1 argument (path), got {}", args.len())
        );
    }
    let path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_dialog_startingpath() expects a string".to_string());
        }
    };
    let mut options = FILE_DIALOG_OPTIONS.lock().unwrap();
    options.startingpath = Some(path);
    Ok(Value::Null)
}

pub fn set_dialog_filters(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!("set_dialog_filters() expects 1 argument (filters), got {}", args.len())
        );
    }
    let filter_arr = match &args[0] {
        Value::Array(arr) => arr,
        _ => {
            return Err("set_dialog_filters() expects an array of filter pairs".to_string());
        }
    };
    let mut filters = Vec::new();
    for filter in filter_arr.iter() {
        if let Value::Array(pair) = &*filter.lock().unwrap() {
            if pair.len() == 2 {
                if
                    let (Value::String(name), Value::Array(exts)) = (
                        &*pair[0].lock().unwrap(),
                        &*pair[1].lock().unwrap(),
                    )
                {
                    let extensions: Vec<String> = exts
                        .iter()
                        .map(|e| e.lock().unwrap().to_string())
                        .collect();
                    filters.push((name.clone(), extensions));
                } else {
                    return Err("Each filter pair must be [string, array of strings]".to_string());
                }
            } else {
                return Err("Each filter must be a pair [name, extensions]".to_string());
            }
        } else {
            return Err("Filters must be an array of [name, extensions] pairs".to_string());
        }
    }
    let mut options = FILE_DIALOG_OPTIONS.lock().unwrap();
    options.filters = Some(filters);
    Ok(Value::Null)
}

pub fn get_dialog_title(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("get_dialog_title() expects 0 arguments, got {}", args.len()));
    }
    let options = FILE_DIALOG_OPTIONS.lock().unwrap();
    Ok(match &options.title {
        Some(title) => Value::String(title.clone()),
        None => Value::Null,
    })
}

pub fn get_dialog_multiselect(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("get_dialog_multiselect() expects 0 arguments, got {}", args.len()));
    }
    let options = FILE_DIALOG_OPTIONS.lock().unwrap();
    Ok(Value::Bool(options.multiselect))
}

pub fn get_dialog_startingpath(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("get_dialog_startingpath() expects 0 arguments, got {}", args.len()));
    }
    let options = FILE_DIALOG_OPTIONS.lock().unwrap();
    Ok(match &options.startingpath {
        Some(path) => Value::String(path.clone()),
        None => Value::Null,
    })
}

pub fn get_dialog_filters(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("get_dialog_filters() expects 0 arguments, got {}", args.len()));
    }
    let options = FILE_DIALOG_OPTIONS.lock().unwrap();
    match &options.filters {
        Some(filters) => {
            let filter_pairs: Vec<Arc<Mutex<Value>>> = filters
                .iter()
                .map(|(name, exts)| {
                    let exts_vec: Vec<Arc<Mutex<Value>>> = exts
                        .iter()
                        .map(|ext| Arc::new(Mutex::new(Value::String(ext.clone()))))
                        .collect();
                    Arc::new(
                        Mutex::new(
                            Value::Array(
                                vec![
                                    Arc::new(Mutex::new(Value::String(name.clone()))),
                                    Arc::new(Mutex::new(Value::Array(exts_vec)))
                                ]
                            )
                        )
                    )
                })
                .collect();
            Ok(Value::Array(filter_pairs))
        }
        None => Ok(Value::Null),
    }
}
// Keep get_dialog_result as is
pub fn get_dialog_result(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!("get_dialog_result() expects 1 argument (dialog_id), got {}", args.len())
        );
    }
    let dialog_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_dialog_result() expects a dialog identifier".to_string());
        }
    };
    let results = DIALOG_RESULTS.read().unwrap();
    match results.get(&dialog_id) {
        Some(Some(path)) => Ok(Value::String(path.clone())),
        Some(None) => Ok(Value::Null), // Dialog closed with no selection
        None => Ok(Value::String(format!("result_pending_{}", dialog_id))),
    }
}

pub fn show_message_box(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 || args.len() > 4 {
        return Err(
            format!(
                "show_message_box() expects 1-4 arguments (message, [title, buttons, icon]), got {}",
                args.len()
            )
        );
    }

    let message = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("show_message_box() expects a string for message".to_string());
        }
    };

    let title = args.get(1).and_then(|v| {
        match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    });
    let buttons = args.get(2).and_then(|v| {
        match v {
            Value::String(s) =>
                match s.to_lowercase().as_str() {
                    "ok" => Some(MsgBoxButtons::Ok),
                    "yesno" => Some(MsgBoxButtons::YesNo),
                    "yesnocancel" => Some(MsgBoxButtons::YesNoCancel),
                    _ => None,
                }
            _ => None,
        }
    });
    let icon = args.get(3).and_then(|v| {
        match v {
            Value::String(s) =>
                match s.to_lowercase().as_str() {
                    "success" => Some(MsgBoxIcon::Success),
                    "error" => Some(MsgBoxIcon::Error),
                    "info" => Some(MsgBoxIcon::Info),
                    "warning" => Some(MsgBoxIcon::Warning),
                    _ => None,
                }
            _ => None,
        }
    });

    let msgbox_id = uuid::Uuid::new_v4().to_string();
    let mut msgbox = MsgBox::new(message, title, buttons, icon);
    msgbox.open();

    let mut msgboxes = MSGBOXES.write().unwrap();
    msgboxes.insert(msgbox_id.clone(), msgbox);

    Ok(Value::String(msgbox_id))
}

pub fn get_message_box_response(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!("get_message_box_response() expects 1 argument (msgbox_id), got {}", args.len())
        );
    }

    let msgbox_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err(
                "get_message_box_response() expects a message box identifier (string)".to_string()
            );
        }
    };

    let max_retries = 5; // Number of retries before giving up
    let retry_delay = Duration::from_millis(500); // Delay between retries

    for _ in 0..max_retries {
        {
            let mut msgboxes = MSGBOXES.write().unwrap(); // Lock the message boxes
            if let Some(msgbox) = msgboxes.get_mut(&msgbox_id) {
                if let Some(response) = &msgbox.response {
                    msgbox.response_retrieved = true; // Mark as retrieved
                    return Ok(Value::String(response.clone()));
                }
            } else {
                return Err("Message box not found".to_string());
            }
        }
        // Sleep before retrying
        thread::sleep(retry_delay);
    }

    Ok(Value::Null) // Return Null if no response after retries
}

pub fn set_datetime(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_datetime() expects 2 arguments, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_datetime() expects a control identifier".to_string());
        }
    };
    let datetime_str = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_datetime() expects a datetime string".to_string());
        }
    };

    let mut states = DATETIMEPICKER_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&control_id) {
        if let Ok(dt) = DateTime::parse_from_str(&datetime_str, &state.format) {
            state.selected_datetime = dt.with_timezone(&Local);
            let mut controls = CONTROLS.write().unwrap();
            if let Some(control) = controls.get_mut(&control_id) {
                control.text = state.selected_datetime.format(&state.format).to_string();
            }
            Ok(Value::Null)
        } else {
            Err("Invalid datetime format".to_string())
        }
    } else {
        Err("DateTimePicker not found".to_string())
    }
}

pub fn get_datetime(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_datetime() expects 1 argument, got {}", args.len()));
    }
    let control_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_datetime() expects a control identifier".to_string());
        }
    };
    let states = DATETIMEPICKER_STATES.read().unwrap();
    if let Some(state) = states.get(&control_id) {
        Ok(Value::String(state.selected_datetime.format(&state.format).to_string()))
    } else {
        Err("DateTimePicker not found".to_string())
    }
}

pub fn set_timerpicker_time(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!(
                "set_timerpicker_time() expects 2 arguments (timerpicker_id, time_string), got {}",
                args.len()
            )
        );
    }

    let timerpicker_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_timerpicker_time() expects a timerpicker identifier".to_string());
        }
    };

    let time_string = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_timerpicker_time() expects a time string".to_string());
        }
    };

    let mut states = TIMERPICKER_STATES.lock().unwrap();
    if let Some(state) = states.get_mut(&timerpicker_id) {
        // Parse the time string using the stored format
        match NaiveTime::parse_from_str(&time_string, &state.format) {
            Ok(time) => {
                state.selected_time = time;
                let mut controls = CONTROLS.write().unwrap();
                if let Some(ctrl) = controls.get_mut(&timerpicker_id) {
                    ctrl.text = time.format(&state.format).to_string();
                }
                Ok(Value::Null)
            }
            Err(e) => Err(format!("Failed to parse time '{}': {}", time_string, e)),
        }
    } else {
        Err("Timerpicker not found".to_string())
    }
}

pub fn get_timerpicker_time(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!(
                "get_timerpicker_time() expects 1 argument (timerpicker_id), got {}",
                args.len()
            )
        );
    }

    let timerpicker_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_timerpicker_time() expects a timerpicker identifier".to_string());
        }
    };

    let states = TIMERPICKER_STATES.lock().unwrap();
    if let Some(state) = states.get(&timerpicker_id) {
        Ok(Value::String(state.selected_time.format(&state.format).to_string()))
    } else {
        Err("Timerpicker not found".to_string())
    }
}

pub fn set_table_data(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            format!(
                "set_table_data() expects 3 arguments (table_id, headers, rows), got {}",
                args.len()
            )
        );
    }
    let table_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("set_table_data() expects a table identifier".to_string());
        }
    };
    let headers = match &args[1] {
        Value::Array(arr) =>
            arr
                .iter()
                .map(|v| v.lock().unwrap().to_string())
                .collect::<Vec<String>>(),
        _ => {
            return Err("set_table_data() expects an array of headers".to_string());
        }
    };
    let rows = match &args[2] {
        Value::Array(arr) => {
            let mut rows_vec = Vec::new();
            for row in arr.iter() {
                match &*row.lock().unwrap() {
                    Value::Array(row_arr) => {
                        let cells = row_arr
                            .iter()
                            .map(|cell| cell.lock().unwrap().to_string())
                            .collect::<Vec<String>>();
                        if cells.len() != headers.len() {
                            return Err(
                                format!(
                                    "Row has {} cells, but expected {} to match headers",
                                    cells.len(),
                                    headers.len()
                                )
                            );
                        }
                        rows_vec.push(cells);
                    }
                    _ => {
                        return Err("Each row must be an array of cell values".to_string());
                    }
                }
            }
            rows_vec
        }
        _ => {
            return Err("set_table_data() expects an array of rows".to_string());
        }
    };

    let mut states = TABLE_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&table_id) {
        state.headers = headers;
        state.rows = rows;
        Ok(Value::Null)
    } else {
        Err("Table not found".to_string())
    }
}

pub fn get_table_data(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_table_data() expects 1 argument, got {}", args.len()));
    }
    let table_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("get_table_data() expects a table identifier".to_string());
        }
    };
    let states = TABLE_STATES.read().unwrap();
    if let Some(state) = states.get(&table_id) {
        let headers = state.headers
            .iter()
            .map(|h| Arc::new(Mutex::new(Value::String(h.clone()))))
            .collect();
        let rows = state.rows
            .iter()
            .map(|row| {
                let row_vec = row
                    .iter()
                    .map(|cell| Arc::new(Mutex::new(Value::String(cell.clone()))))
                    .collect();
                Arc::new(Mutex::new(Value::Array(row_vec)))
            })
            .collect();
        Ok(
            Value::Array(
                vec![
                    Arc::new(Mutex::new(Value::Array(headers))),
                    Arc::new(Mutex::new(Value::Array(rows)))
                ]
            )
        )
    } else {
        Err("Table not found".to_string())
    }
}

// Updated add_menu_item now accepts an optional icon parameter as the fifth argument
pub fn add_menu_item(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 5 {
        return Err(
            format!(
                "add_menu_item() expects 2-5 arguments (menu_id, label, [callback], [children], [icon]), got {}",
                args.len()
            )
        );
    }

    let menu_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("add_menu_item() expects a menu identifier".to_string());
        }
    };
    let label = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("add_menu_item() expects a label string".to_string());
        }
    };

    let callback = args.get(2).cloned();

    // Recursively handle submenu arrays
    let children = match args.get(3) {
        Some(Value::Array(arr)) =>
            arr
                .iter()
                .map(|child| parse_menu_item(&*child.lock().unwrap()))
                .collect(),
        _ => vec![],
    };

    // Optional icon parameter (expects a string for the icon's file path or identifier)
    let icon = match args.get(4) {
        Some(Value::String(icon_str)) => Some(icon_str.clone()),
        _ => None,
    };

    let mut states = MENU_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&menu_id) {
        state.items.push(MenuItem {
            label,
            icon, // Set the icon field
            callback,
            children,
            is_separator: false,
        });
        Ok(Value::Null)
    } else {
        Err("Menu not found".to_string())
    }
}

// Update add_submenu_item similarly to accept the icon if needed (here we assume the same order)
pub fn add_submenu_item(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 3 || args.len() > 7 {
        return Err(
            format!(
                "add_submenu_item() expects 3-7 arguments (menu_id, menu_item_name, submenu_name, [callback], [children], [separator], [icon]), got {}",
                args.len()
            )
        );
    }

    let menu_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("add_submenu_item() expects a menu identifier".to_string());
        }
    };

    let menu_item_name = match &args[1] {
        Value::String(name) => name.clone(),
        _ => {
            return Err("add_submenu_item() expects a menu item name".to_string());
        }
    };

    let submenu_name = match &args[2] {
        Value::String(name) => name.clone(),
        _ => {
            return Err("add_submenu_item() expects a submenu name".to_string());
        }
    };

    let callback = args.get(3).cloned();

    let children = match args.get(4) {
        Some(Value::Array(arr)) =>
            arr
                .iter()
                .map(|child| parse_menu_item(&*child.lock().unwrap()))
                .collect(),
        _ => vec![],
    };

    let is_separator = match args.get(5) {
        Some(Value::Bool(b)) => *b,
        _ => false,
    };

    // Optional icon parameter for the submenu (7th argument)
    let icon = match args.get(6) {
        Some(Value::String(icon_str)) => Some(icon_str.clone()),
        _ => None,
    };

    let mut states = MENU_STATES.write().unwrap();
    if let Some(state) = states.get_mut(&menu_id) {
        // Recursively find the menu item and add the submenu with the icon field
        if
            find_and_add_submenu(
                &mut state.items,
                &menu_item_name,
                submenu_name,
                callback,
                children,
                is_separator,
                icon
            )
        {
            Ok(Value::Null)
        } else {
            Err(format!("Menu item '{}' not found", menu_item_name))
        }
    } else {
        Err("Menu not found".to_string())
    }
}

// Updated recursive helper to pass the icon along
fn find_and_add_submenu(
    items: &mut Vec<MenuItem>,
    menu_item_name: &str,
    submenu_name: String,
    callback: Option<Value>,
    children: Vec<MenuItem>,
    is_separator: bool,
    icon: Option<String>
) -> bool {
    for item in items.iter_mut() {
        if item.label == menu_item_name {
            item.children.push(MenuItem {
                label: submenu_name,
                icon, // Set icon for submenu if provided
                callback,
                children,
                is_separator,
            });
            return true;
        }
        if
            find_and_add_submenu(
                &mut item.children,
                menu_item_name,
                submenu_name.clone(),
                callback.clone(),
                children.clone(),
                is_separator,
                icon.clone()
            )
        {
            return true;
        }
    }
    false
}

fn find_menu_item_mut<'a>(items: &'a mut [MenuItem], label: &str) -> Option<&'a mut MenuItem> {
    for item in items {
        if item.label == label {
            return Some(item);
        }
        if let Some(found) = find_menu_item_mut(&mut item.children, label) {
            return Some(found);
        }
    }
    None
}

/// Recursively searches for a menu item by label in an immutable slice.
/// Returns an immutable reference if found.
fn find_menu_item<'a>(items: &'a [MenuItem], label: &str) -> Option<&'a MenuItem> {
    for item in items {
        if item.label == label {
            return Some(item);
        }
        if let Some(found) = find_menu_item(&item.children, label) {
            return Some(found);
        }
    }
    None
}

// Expects arguments: [menu_id: String, menu_item_label: String, icon: String or Null]
pub fn set_menu_item_icon(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            "set_menu_item_icon expects 3 arguments: menu_id, menu_item_label, icon".to_string()
        );
    }

    // Parse menu_id argument.
    let menu_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("menu_id must be a string".to_string());
        }
    };

    // Parse menu item label.
    let item_label = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("menu_item_label must be a string".to_string());
        }
    };

    // Parse the icon argument: allow a String (to set an icon) or Null (to remove it).
    let icon = match &args[2] {
        Value::String(s) => Some(s.clone()),
        Value::Null => None,
        _ => {
            return Err("icon must be a string or null".to_string());
        }
    };

    // Lock the global menu state for writing.
    let mut states = MENU_STATES.write().unwrap();
    let state = states.get_mut(&menu_id).ok_or("Menu not found")?;

    // Recursively find the menu item by its label.
    let item = find_menu_item_mut(&mut state.items, &item_label).ok_or("Menu item not found")?;

    // Set the icon value.
    item.icon = icon;

    Ok(Value::Null)
}

// --- Getter function ---
// Expects arguments: [menu_id: String, menu_item_label: String]
pub fn get_menu_item_icon(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("get_menu_item_icon expects 2 arguments: menu_id, menu_item_label".to_string());
    }

    // Parse menu_id argument.
    let menu_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("menu_id must be a string".to_string());
        }
    };

    // Parse menu item label.
    let item_label = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("menu_item_label must be a string".to_string());
        }
    };

    // Lock the global menu state for reading.
    let states = MENU_STATES.read().unwrap();
    let state = states.get(&menu_id).ok_or("Menu not found")?;

    // Recursively find the menu item.
    let item = find_menu_item(&state.items, &item_label).ok_or("Menu item not found")?;

    // Return the icon value as a Value::String if present, or Value::Null if not.
    match &item.icon {
        Some(icon_str) => Ok(Value::String(icon_str.clone())),
        None => Ok(Value::Null),
    }
}

pub fn add_separator_item(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("add_separator_item() does not accept any arguments".to_string());
    }

    let mut states = MENU_STATES.write().unwrap();

    // Find the last modified menu that has more than one item
    let mut last_menu = None;
    for state in states.values_mut() {
        if state.items.len() > 1 {
            last_menu = Some(state);
        }
    }

    if let Some(state) = last_menu {
        state.items.push(MenuItem {
            label: String::new(), // Empty label for separator
            callback: None,
            children: vec![], // No children for separators
            is_separator: true,
            icon: None, // Mark as separator
        });
        return Ok(Value::Null);
    }

    Err("No valid menu found or not enough items to insert a separator".to_string())
}

// Update the parse_menu_item helper to optionally parse an icon from a fifth element in the array.
fn parse_menu_item(value: &Value) -> MenuItem {
    match value {
        Value::Array(arr) => {
            // Extract label
            let label = match arr.get(0) {
                Some(child_value) =>
                    match child_value.lock() {
                        Ok(inner_value) =>
                            match &*inner_value {
                                Value::String(s) => s.clone(),
                                _ => "Invalid label".to_string(),
                            }
                        Err(_) => "Failed to lock".to_string(),
                    }
                _ => "Invalid label".to_string(),
            };

            // Callback as second item
            let callback = match arr.get(1) {
                Some(callback_value) =>
                    match callback_value.lock() {
                        Ok(inner_value) => Some(inner_value.clone()),
                        Err(_) => None,
                    }
                None => None,
            };

            // Children (third item)
            let children = match arr.get(2) {
                Some(child_value) =>
                    match child_value.lock() {
                        Ok(inner_value) => {
                            if let Value::Array(child_arr) = &*inner_value {
                                child_arr
                                    .iter()
                                    .map(|c| parse_menu_item(&*c.lock().unwrap()))
                                    .collect()
                            } else {
                                vec![]
                            }
                        }
                        Err(_) => vec![],
                    }
                _ => vec![],
            };

            // Separator flag (fourth item)
            let is_separator = match arr.get(3) {
                Some(separator_value) =>
                    match separator_value.lock() {
                        Ok(inner_value) => {
                            if let Value::Bool(b) = &*inner_value { *b } else { false }
                        }
                        Err(_) => false,
                    }
                _ => false,
            };

            // New: Optional icon field (fifth item)
            let icon = match arr.get(4) {
                Some(icon_value) =>
                    match icon_value.lock() {
                        Ok(inner_value) => {
                            if let Value::String(icon_str) = &*inner_value {
                                Some(icon_str.clone())
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                _ => None,
            };

            MenuItem {
                label,
                icon,
                callback,
                children,
                is_separator,
            }
        }
        _ =>
            MenuItem {
                label: "Invalid".to_string(),
                icon: None,
                callback: None,
                children: vec![],
                is_separator: false,
            },
    }
}

fn render_menu_items(
    ui: &mut egui::Ui,
    items: &[MenuItem],
    state: &mut MenuState,
    ctx: &egui::Context,
    control: &ControlSettings
) {
    // Update style as before
    let mut style = ui.style_mut();
    style.visuals.widgets.inactive.bg_fill = control.backcolor;
    style.visuals.widgets.hovered.bg_fill = lighten(control.backcolor, 0.1);
    style.visuals.widgets.active.bg_fill = lighten(control.backcolor, 0.2);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, control.forecolor);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, control.forecolor);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, control.forecolor);
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(control.fontsize, get_font_family(&control.fontname))
    );

    for item in items {
        // Render a horizontal group for each menu item.
        if item.children.is_empty() {
            ui.horizontal(|ui| {
                // Always reserve space for the icon (16px icon + 4px spacing).
                if let Some(icon_path) = &item.icon {
                    if let Some(icon_texture) = get_icon_texture(ctx, icon_path) {
                        ui.add(
                            egui::Image
                                ::new(&icon_texture)
                                .fit_to_exact_size(egui::vec2(16.0, 16.0))
                        );
                        ui.add_space(4.0);
                    } else {
                        // If icon_path provided but texture not loaded, allocate same space.
                        ui.allocate_space(egui::vec2(16.0 + 4.0, 16.0));
                    }
                } else {
                    // No icon provided: reserve the same space so that the labels line up.
                    ui.allocate_space(egui::vec2(16.0 + 4.0, 16.0));
                }

                // Render the menu button.
                if ui.button(&item.label).clicked() {
                    state.is_open = false;
                    if let Some(callback) = &item.callback {
                        if let Value::Function { body, closure, object, .. } = callback {
                            let body = body.clone();
                            let closure = closure.clone();
                            let object = object.clone();
                            std::thread::spawn(move || {
                                let mut interpreter = GLOBAL_INTERPRETER.lock().unwrap();
                                let mut local_env = Arc::new(
                                    Mutex::new(Environment::new(Some(closure)))
                                );
                                if let Some(obj) = object {
                                    let value = obj.lock().unwrap().clone();
                                    local_env.lock().unwrap().define("this".to_string(), value);
                                }
                                let _ = interpreter.visit_block(&body, &mut local_env);
                            });
                            ctx.request_repaint();
                        }
                    }
                }
            });
            // Add a little vertical space between items.
            ui.add_space(2.0);
        } else {
            // Render a menu button with children (submenu).
            ui.menu_button(&item.label, |ui| {
                let child_items = item.children.clone();
                render_menu_items(ui, &child_items, state, ctx, control);
            });
            ui.add_space(2.0);
        }
        if item.is_separator {
            ui.vertical(|ui| {
                ui.separator();
            });
        }
    }
}

/// Loads an icon texture from a file if it hasn't been loaded before,
/// then caches and returns it. Returns None if loading fails.
fn get_icon_texture(ctx: &Context, icon_path: &str) -> Option<TextureHandle> {
    // Check the cache first.
    {
        let cache = ICON_CACHE.lock().unwrap();
        if let Some(texture) = cache.get(icon_path) {
            return Some(texture.clone());
        }
    }

    // Attempt to load the file from disk.
    let image_data = std::fs::read(icon_path).ok()?;
    // Decode the image using the image crate.
    let image = image::load_from_memory(&image_data).ok()?;
    let image = image.to_rgba8();
    let dimensions = image.dimensions();
    let size = [dimensions.0 as usize, dimensions.1 as usize];
    let pixels = image.into_raw();

    // Create an egui ColorImage from the decoded pixel data.
    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

    // Load the texture into egui.
    let texture = ctx.load_texture(icon_path, color_image, TextureOptions::default());

    // Cache the texture.
    let mut cache = ICON_CACHE.lock().unwrap();
    cache.insert(icon_path.to_string(), texture.clone());

    Some(texture)
}

pub fn setseparatororientation(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            "setseparatororientation() expects 2 arguments (separator_id, orientation)".to_string()
        );
    }

    let separator_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a separator identifier".to_string());
        }
    };

    let orientation = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("Second argument must be a string (orientation)".to_string());
        }
    };

    let mut controls = CONTROLS.write().unwrap();
    if let Some(settings) = controls.get_mut(&separator_id) {
        if settings.control_type == "separator" {
            settings.orientation = orientation;
            Ok(Value::Null)
        } else {
            Err("Control is not a separator".to_string())
        }
    } else {
        Err("Separator not found".to_string())
    }
}

pub fn getseparatororientation(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("getseparatororientation() expects 1 argument (separator_id)".to_string());
    }

    let separator_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a separator identifier".to_string());
        }
    };

    let controls = CONTROLS.read().unwrap();
    if let Some(settings) = controls.get(&separator_id) {
        if settings.control_type == "separator" {
            Ok(Value::String(settings.orientation.clone()))
        } else {
            Err("Control is not a separator".to_string())
        }
    } else {
        Err("Separator not found".to_string())
    }
}

pub fn set_gridline_color(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("set_grid_line_color() expects 2 arguments (layout_id, color)".to_string());
    }
    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a layout identifier".to_string());
        }
    };
    let color = match &args[1] {
        Value::String(s) => parse_color(s).unwrap_or(Color32::GRAY),
        _ => {
            return Err("Second argument must be a color string (e.g., '255,0,0')".to_string());
        }
    };
    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    if let Some(state) = grid_states.get_mut(&layout_id) {
        state.line_color = color;
        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn set_gridline_thickness(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            "set_grid_line_thickness() expects 2 arguments (layout_id, thickness)".to_string()
        );
    }
    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a layout identifier".to_string());
        }
    };
    let thickness = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Second argument must be a number".to_string());
        }
    };
    if thickness < 0.0 {
        return Err("Thickness must be non-negative".to_string());
    }
    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    if let Some(state) = grid_states.get_mut(&layout_id) {
        state.line_thickness = thickness;
        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn setgridlayoutshowlines(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("setgridlayoutshowlines() expects 2 arguments (layout_id, show)".to_string());
    }
    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a layout identifier".to_string());
        }
    };
    let show = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("Second argument must be a boolean".to_string());
        }
    };
    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    if let Some(state) = grid_states.get_mut(&layout_id) {
        state.show_grid_lines = show;
        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn getgridlayoutshowlines(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("getgridlayoutshowlines() expects 1 argument (layout_id)".to_string());
    }
    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a layout identifier".to_string());
        }
    };
    let grid_states = GRIDLAYOUT_STATES.read().unwrap();
    if let Some(state) = grid_states.get(&layout_id) {
        Ok(Value::Bool(state.show_grid_lines))
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn setgridlayoutrows(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("setgridlayoutrows() expects 2 arguments (layout_id, rows)".to_string());
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a layout identifier".to_string());
        }
    };
    let rows = match &args[1] {
        Value::Number(n) => {
            let r = *n as usize;
            if r < 1 {
                return Err("Rows must be at least 1".to_string());
            }
            r
        }
        _ => {
            return Err("Second argument must be a number (rows)".to_string());
        }
    };

    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    if let Some(state) = grid_states.get_mut(&layout_id) {
        // Resize cell_controls to match the new number of rows
        if rows > state.cell_controls.len() {
            let additional = rows - state.cell_controls.len();
            for _ in 0..additional {
                state.cell_controls.push(vec![None; state.cols]);
            }
        } else if rows < state.cell_controls.len() {
            state.cell_controls.truncate(rows);
        }
        state.rows = rows;
        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn getgridlayoutrows(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("getgridlayoutrows() expects 1 argument (layout_id)".to_string());
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a layout identifier".to_string());
        }
    };

    let grid_states = GRIDLAYOUT_STATES.read().unwrap();
    if let Some(state) = grid_states.get(&layout_id) {
        Ok(Value::Number(state.rows as f64))
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn setgridlayoutcolumns(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("setgridlayoutcolumns() expects 2 arguments (layout_id, columns)".to_string());
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a layout identifier".to_string());
        }
    };
    let columns = match &args[1] {
        Value::Number(n) => {
            let c = *n as usize;
            if c < 1 {
                return Err("Columns must be at least 1".to_string());
            }
            c
        }
        _ => {
            return Err("Second argument must be a number (columns)".to_string());
        }
    };

    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    if let Some(state) = grid_states.get_mut(&layout_id) {
        // Resize each row in cell_controls to match the new number of columns
        for row in &mut state.cell_controls {
            if columns > row.len() {
                let additional = columns - row.len();
                for _ in 0..additional {
                    row.push(None);
                }
            } else if columns < row.len() {
                row.truncate(columns);
            }
        }
        state.cols = columns;
        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn getgridlayoutcolumns(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("getgridlayoutcolumns() expects 1 argument (layout_id)".to_string());
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a layout identifier".to_string());
        }
    };

    let grid_states = GRIDLAYOUT_STATES.read().unwrap();
    if let Some(state) = grid_states.get(&layout_id) {
        Ok(Value::Number(state.cols as f64))
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn addgridlayoutrow(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("addgridlayoutrow() expects 1 argument (layout_id)".to_string());
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a layout identifier".to_string());
        }
    };

    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    if let Some(state) = grid_states.get_mut(&layout_id) {
        let new_row = vec![None; state.cols]; // New row with 'cols' empty cells
        state.cell_controls.push(new_row);
        state.rows += 1;
        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn addgridlayoutcolumn(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("addgridlayoutcolumn() expects 1 argument (layout_id)".to_string());
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Argument must be a layout identifier".to_string());
        }
    };

    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    if let Some(state) = grid_states.get_mut(&layout_id) {
        for row in &mut state.cell_controls {
            row.push(None); // Add an empty cell to each row
        }
        state.cols += 1;
        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn setgridlayoutcell(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(
            "setgridlayoutcell() expects 4 arguments (layout_id, row, col, control_id)".to_string()
        );
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a layout identifier".to_string());
        }
    };
    let row = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("Second argument must be a number (row)".to_string());
        }
    };
    let col = match &args[2] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("Third argument must be a number (col)".to_string());
        }
    };
    let control_id = match &args[3] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("Fourth argument must be a control identifier".to_string());
        }
    };

    let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
    let mut controls = CONTROLS.write().unwrap();
    let mut forms = FORMS.write().unwrap();

    if let Some(state) = grid_states.get_mut(&layout_id) {
        if row >= state.rows || col >= state.cols {
            return Err("Row or column index out of bounds".to_string());
        }

        // Ensure the grid's cell_controls is properly sized
        while state.cell_controls.len() < state.rows {
            state.cell_controls.push(vec![None; state.cols]);
        }
        for r in &mut state.cell_controls {
            while r.len() < state.cols {
                r.push(None);
            }
        }

        // Remove existing control from children if any
        if let Some(existing_id) = state.cell_controls[row][col].take() {
            if let Some(settings) = controls.get_mut(&layout_id) {
                if let Some(pos) = settings.children.iter().position(|x| *x == existing_id) {
                    settings.children.remove(pos);
                }
            }
        }

        // Set new control in the grid cell
        state.cell_controls[row][col] = Some(control_id.clone());

        // Add to grid's children if not already present
        if let Some(settings) = controls.get_mut(&layout_id) {
            if !settings.children.contains(&control_id) {
                settings.children.push(control_id.clone());
            }

            // Remove the control from the form's controls_order to prevent double rendering
            let form_id = settings.form_id.clone();
            if let Some(form) = forms.get_mut(&form_id) {
                form.controls_order.retain(|id| id != &control_id);
            }
        }

        Ok(Value::Null)
    } else {
        Err("GridLayout not found".to_string())
    }
}

pub fn setspacing(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!("setspacing() expects 2 arguments (layout_id, spacing), got {}", args.len())
        );
    }

    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("First argument must be a layout identifier (string)".to_string());
        }
    };

    let spacing = match &args[1] {
        Value::Number(n) => *n as f32,
        _ => {
            return Err("Second argument must be a number (spacing)".to_string());
        }
    };

    // Try FlowLayout (no check for negative spacing here)
    {
        let mut flow_states = FLOWLAYOUT_STATES.write().unwrap();
        if let Some(state) = flow_states.get_mut(&layout_id) {
            state.spacing = spacing;
            return Ok(Value::Null);
        }
    }

    // Try HorizontalLayout (spacing must be non-negative)
    {
        let mut hlayout_states = HORIZONTALLAYOUT_STATES.write().unwrap();
        if let Some(state) = hlayout_states.get_mut(&layout_id) {
            if spacing < 0.0 {
                return Err("Spacing must be non-negative".to_string());
            }
            state.spacing = spacing;
            return Ok(Value::Null);
        }
    }

    // Try VerticalLayout (spacing must be non-negative)
    {
        let mut vlayout_states = VERTICALLAYOUT_STATES.write().unwrap();
        if let Some(state) = vlayout_states.get_mut(&layout_id) {
            if spacing < 0.0 {
                return Err("Spacing must be non-negative".to_string());
            }
            state.spacing = spacing;
            return Ok(Value::Null);
        }
    }

    Err(
        "Layout not found. Supported controls: FlowLayout, HorizontalLayout, VerticalLayout".to_string()
    )
}

pub fn getspacing(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("getspacing() expects 1 argument (layout_id)".to_string());
    }

    // Extract the layout identifier as a string
    let layout_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("getspacing() expects a layout identifier (string)".to_string());
        }
    };

    {
        let flow_states = FLOWLAYOUT_STATES.read().unwrap();
        if let Some(state) = flow_states.get(&layout_id) {
            return Ok(Value::Number(state.spacing as f64));
        }
    }
    {
        let hlayout_states = HORIZONTALLAYOUT_STATES.read().unwrap();
        if let Some(state) = hlayout_states.get(&layout_id) {
            return Ok(Value::Number(state.spacing as f64));
        }
    }
    {
        let vlayout_states = VERTICALLAYOUT_STATES.read().unwrap();
        if let Some(state) = vlayout_states.get(&layout_id) {
            return Ok(Value::Number(state.spacing as f64));
        }
    }

    Err(
        "Layout not found. Supported layouts: FlowLayout, HorizontalLayout, VerticalLayout".to_string()
    )
}

pub fn render_control(
    ui: &mut Ui,
    control_id: &String,
    control: &ControlSettings,
    ctx: &egui::Context,
    textbox_texts: &mut HashMap<String, String>
) {
    // Wrap all rendering logic in ui.push_id to scope widget IDs with control_id
    ui.push_id(control_id, |ui| {
        // Calculate position and size based on docking and autosizing
        let (pos, size) = if matches!(control.dock, DockStyle::None) {
            (
                control.position + Vec2::new(control.margin.0, control.margin.1),
                Vec2::new(control.width, control.height),
            )
        } else {
            let form_rect = ui.clip_rect();
            let (text_width, text_height) = if control.autosize {
                ui_text_size(ui, &control.text, control)
            } else {
                (0.0, 0.0)
            };
            match control.dock {
                DockStyle::Top => {
                    let height = if control.autosize {
                        text_height + control.padding.1 + control.padding.3
                    } else {
                        control.height
                    };
                    (form_rect.min, Vec2::new(form_rect.width(), height))
                }
                DockStyle::Bottom => {
                    let height = if control.autosize {
                        text_height + control.padding.1 + control.padding.3
                    } else {
                        control.height
                    };
                    (
                        Pos2::new(form_rect.min.x, form_rect.max.y - height),
                        Vec2::new(form_rect.width(), height),
                    )
                }
                DockStyle::Left => {
                    let width = if control.autosize {
                        text_width + control.padding.0 + control.padding.2
                    } else {
                        control.width
                    };
                    (form_rect.min, Vec2::new(width, form_rect.height()))
                }
                DockStyle::Right => {
                    let width = if control.autosize {
                        text_width + control.padding.0 + control.padding.2
                    } else {
                        control.width
                    };
                    (
                        Pos2::new(form_rect.max.x - width, form_rect.min.y),
                        Vec2::new(width, form_rect.height()),
                    )
                }
                DockStyle::Fill => (form_rect.min, form_rect.size()),
                DockStyle::None => unreachable!(),
            }
        };

        let rect = Rect::from_min_size(pos, size);
        let current_time = ctx.input(|i| i.time);
        let painter = ui.painter();
        let controls = CONTROLS.read().unwrap();
        // Render based on control type
        match control.control_type.as_str() {
            "slider" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut states = SLIDER_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(rect, |ui| {
                        let slider = egui::Slider::new(&mut state.value, state.min..=state.max);
                        let slider = if state.orientation == SliderOrientation::Vertical {
                            slider.vertical()
                        } else {
                            slider
                        };
                        ui.add(slider);
                    });
                }
            }
            "treeview" => {
                // Render the treeview background and border.
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                if control.border {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::GRAY),
                        StrokeKind::Inside
                    );
                }
                let mut states = TREEVIEW_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(rect, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // The recursive rendering function which now uses a map of event-specific callbacks.
                            fn render_nodes(
                                ui: &mut Ui,
                                nodes: &mut Vec<TreeNode>,
                                selected_node: &mut Option<String>,
                                events: &HashMap<String, Option<Value>>,
                                ctx: &egui::Context,
                                control_id: &String
                            ) {
                                for node in nodes.iter_mut() {
                                    let id = Id::new(&node.id);
                                    ui.push_id(id, |ui| {
                                        ui.horizontal(|ui| {
                                            // Render an icon if one is specified.
                                            if let Some(icon_path) = &node.icon {
                                                if
                                                    let Some(icon_texture) = get_icon_texture(
                                                        ctx,
                                                        icon_path
                                                    )
                                                {
                                                    ui.add(
                                                        egui::Image
                                                            ::new(&icon_texture)
                                                            .fit_to_exact_size(
                                                                egui::vec2(16.0, 16.0)
                                                            )
                                                    );
                                                    ui.add_space(2.0);
                                                }
                                            }
                                            // Render a checkbox if it is defined.
                                            if let Some(checked) = &mut node.checkbox {
                                                let checkbox_response = ui.checkbox(checked, "");
                                                if checkbox_response.changed() {
                                                    // node_to_value returns a dictionary now.
                                                    let node_value = node_to_value(node);
                                                    let evt = "checkboxchange".to_string();
                                                    if let Some(Some(callback)) = events.get(&evt) {
                                                        call_on_event(
                                                            callback,
                                                            Value::String(evt),
                                                            node_value
                                                        );
                                                    }
                                                }
                                            }
                                            // Render the node: collapsible header if there are children, otherwise a selectable label.
                                            let response = if !node.children.is_empty() {
                                                let collapsing = ui.collapsing(
                                                    node.text.clone(),
                                                    |ui| {
                                                        render_nodes(
                                                            ui,
                                                            &mut node.children,
                                                            selected_node,
                                                            events,
                                                            ctx,
                                                            control_id
                                                        );
                                                    }
                                                );
                                                if collapsing.header_response.clicked() {
                                                    node.expanded = !node.expanded;
                                                }
                                                collapsing.header_response
                                            } else {
                                                let label_response = ui.selectable_label(
                                                    selected_node.as_ref() == Some(&node.id),
                                                    &node.text
                                                );
                                                if label_response.clicked() {
                                                    *selected_node = Some(node.id.clone());
                                                }
                                                label_response
                                            };

                                            // Process double-click events.
                                            if response.double_clicked() {
                                                *selected_node = Some(node.id.clone());
                                                let node_value = node_to_value(node);
                                                let evt = "doubleclick".to_string();
                                                if let Some(Some(callback)) = events.get(&evt) {
                                                    call_on_event(
                                                        callback,
                                                        Value::String(evt),
                                                        node_value
                                                    );
                                                }
                                            }
                                            // Process mouse hover events.
                                            if response.hovered() {
                                                let node_value = node_to_value(node);
                                                let evt = "mousehover".to_string();
                                                if let Some(Some(callback)) = events.get(&evt) {
                                                    call_on_event(
                                                        callback,
                                                        Value::String(evt),
                                                        node_value
                                                    );
                                                }
                                            }
                                            // Process primary mouse button down events.
                                            if response.clicked_by(egui::PointerButton::Primary) {
                                                let node_value = node_to_value(node);
                                                let evt = "mousedown".to_string();
                                                if let Some(Some(callback)) = events.get(&evt) {
                                                    call_on_event(
                                                        callback,
                                                        Value::String(evt),
                                                        node_value
                                                    );
                                                }
                                            }
                                            // Process mouse up events.
                                            if
                                                ui.input(|i| i.pointer.any_released()) &&
                                                response.hovered()
                                            {
                                                let node_value = node_to_value(node);
                                                let evt = "mouseup".to_string();
                                                if let Some(Some(callback)) = events.get(&evt) {
                                                    call_on_event(
                                                        callback,
                                                        Value::String(evt),
                                                        node_value
                                                    );
                                                }
                                            }
                                        });
                                    });
                                }
                            }

                            // Call render_nodes and pass in the events mapping from control settings.
                            render_nodes(
                                ui,
                                &mut state.nodes,
                                &mut state.selected_node,
                                &control.treeview_on_events,
                                ctx,
                                control_id
                            );
                        });
                    });
                }
            }

            "richtext" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                if control.border {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::GRAY),
                        StrokeKind::Outside
                    );
                }
                let mut states = RICHTEXT_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(rect, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut text_edit = egui::TextEdit
                                ::multiline(&mut state.text)
                                .text_color(control.forecolor)
                                .desired_width(rect.width());
                            let response = ui.add(text_edit);
                            if response.changed() {
                                let mut controls = CONTROLS.write().unwrap();
                                if let Some(ctrl) = controls.get_mut(control_id) {
                                    ctrl.text = state.text.clone();
                                }
                            }
                            // Apply formatting (basic example)
                            if !state.formats.is_empty() {
                                let mut job = LayoutJob::default();
                                let mut last_pos = 0;
                                for (start, end, format) in &state.formats {
                                    if last_pos < *start {
                                        job.append(&state.text[last_pos..*start], 0.0, TextFormat {
                                            font_id: FontId::new(14.0, FontFamily::Proportional),
                                            color: control.forecolor,
                                            ..Default::default()
                                        });
                                    }
                                    job.append(&state.text[*start..*end], 0.0, format.clone());
                                    last_pos = *end;
                                }
                                if last_pos < state.text.len() {
                                    job.append(&state.text[last_pos..], 0.0, TextFormat {
                                        font_id: FontId::new(14.0, FontFamily::Proportional),
                                        color: control.forecolor,
                                        ..Default::default()
                                    });
                                }
                                ui.painter().galley(
                                    rect.min,
                                    ui.fonts(|f| f.layout_job(job)),
                                    control.forecolor
                                );
                            }
                        });
                    });
                }
            }
            "toolbar" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut states = TOOLBAR_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(rect, |ui| {
                        ui.horizontal(|ui| {
                            for item in &state.items {
                                // Handle icon button click
                                if let Some(icon_path) = &item.icon {
                                    if let Some(texture) = get_icon_texture(ctx, icon_path) {
                                        let image = egui::Image
                                            ::new(&texture.clone())
                                            .fit_to_exact_size(egui::Vec2::new(24.0, 24.0));
                                        let image_button = egui::ImageButton::new(image);
                                        if ui.add(image_button).clicked() {
                                            if let Some(callback) = &item.callback {
                                                // Correct pattern matching: callback is &Value, no Option wrapping needed
                                                if
                                                    let Value::Function {
                                                        body,
                                                        closure,
                                                        object,
                                                        ..
                                                    } = callback
                                                {
                                                    let body = body.clone();
                                                    let closure = closure.clone();
                                                    let object = object.clone();
                                                    std::thread::spawn(move || {
                                                        let mut interpreter =
                                                            GLOBAL_INTERPRETER.lock().unwrap();
                                                        let mut local_env = Arc::new(
                                                            Mutex::new(
                                                                Environment::new(Some(closure))
                                                            )
                                                        );
                                                        if let Some(obj) = object {
                                                            let value = obj.lock().unwrap().clone();
                                                            let _ = local_env
                                                                .lock()
                                                                .unwrap()
                                                                .define("this".to_string(), value);
                                                        }
                                                        let _ = interpreter.visit_block(
                                                            &body,
                                                            &mut local_env
                                                        );
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }

                                // Handle text button click
                                if ui.button(&item.text).clicked() {
                                    if let Some(callback) = &item.callback {
                                        // Correct pattern matching: callback is &Value, no Option wrapping needed
                                        if
                                            let Value::Function { body, closure, object, .. } =
                                                callback
                                        {
                                            let body = body.clone();
                                            let closure = closure.clone();
                                            let object = object.clone();
                                            std::thread::spawn(move || {
                                                let mut interpreter =
                                                    GLOBAL_INTERPRETER.lock().unwrap();
                                                let mut local_env = Arc::new(
                                                    Mutex::new(Environment::new(Some(closure)))
                                                );
                                                if let Some(obj) = object {
                                                    let value = obj.lock().unwrap().clone();
                                                    local_env
                                                        .lock()
                                                        .unwrap()
                                                        .define("this".to_string(), value);
                                                }
                                                let _ = interpreter.visit_block(
                                                    &body,
                                                    &mut local_env
                                                );
                                            });
                                        }
                                    }
                                }
                            }
                        });
                    });
                }
            }
            "imagebutton" => {
                let mut states = IMAGEBUTTON_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    if let Some(path) = &state.image_path {
                        if state.texture_handle.is_none() {
                            if let Ok(image_data) = fs::read(path) {
                                if let Ok(image) = image::load_from_memory(&image_data) {
                                    let rgba = image.to_rgba8();
                                    let size = [rgba.width() as usize, rgba.height() as usize];
                                    let pixels = rgba.into_raw();
                                    let color_image = ColorImage::from_rgba_unmultiplied(
                                        size,
                                        &pixels
                                    );
                                    state.texture_handle = Some(
                                        ctx.load_texture(
                                            control_id.clone(),
                                            color_image,
                                            Default::default()
                                        )
                                    );
                                }
                            }
                        }
                        if let Some(texture) = &state.texture_handle {
                            ui.allocate_ui_at_rect(rect, |ui| {
                                // Create an Image widget with the texture and set the size
                                let image = egui::Image
                                    ::new(&texture.clone())
                                    .fit_to_exact_size(rect.size()); // Use rect.size() for dynamic sizing
                                let image_button = egui::ImageButton::new(image);
                                let response = ui.add(image_button);
                                if response.clicked() {
                                    if let Some(callback) = &control.callback {
                                        if
                                            let Value::Function { body, closure, object, .. } =
                                                callback
                                        {
                                            let body = body.clone();
                                            let closure = closure.clone();
                                            let object = object.clone();
                                            std::thread::spawn(move || {
                                                let mut interpreter =
                                                    GLOBAL_INTERPRETER.lock().unwrap();
                                                let mut local_env = Arc::new(
                                                    Mutex::new(Environment::new(Some(closure)))
                                                );
                                                if let Some(obj) = object {
                                                    let value = obj.lock().unwrap().clone();
                                                    local_env
                                                        .lock()
                                                        .unwrap()
                                                        .define("this".to_string(), value);
                                                }
                                                let _ = interpreter.visit_block(
                                                    &body,
                                                    &mut local_env
                                                );
                                            });
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
            "statusbar" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let states = STATUSBAR_STATES.read().unwrap();
                if let Some(state) = states.get(control_id) {
                    let text_pos = rect.min + Vec2::new(5.0, 5.0); // Offset slightly for padding
                    let font_id = FontId::new(control.fontsize, get_font_family(&control.fontname));
                    ui.painter().text(
                        text_pos,
                        control.text_alignment,
                        &state.text,
                        font_id,
                        control.forecolor
                    );
                }
            }
            "colordialog" => {
                let mut states = COLORDIALOG_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    let mut is_open_local = state.is_open;
                    let close_request = std::cell::Cell::new(false);

                    if is_open_local {
                        egui::Window
                            ::new("Color Picker")
                            .collapsible(false)
                            .resizable(false)
                            .default_size(egui::vec2(300.0, 400.0)) // Set a reasonable default size
                            .open(&mut is_open_local)
                            .show(ctx, |ui| {
                                // Mode selection tabs
                                ui.horizontal(|ui| {
                                    if
                                        ui
                                            .selectable_label(
                                                matches!(state.mode, ColorDialogMode::Basic),
                                                "Basic"
                                            )
                                            .clicked()
                                    {
                                        state.mode = ColorDialogMode::Basic;
                                    }
                                    if
                                        ui
                                            .selectable_label(
                                                matches!(state.mode, ColorDialogMode::Advanced),
                                                "Advanced"
                                            )
                                            .clicked()
                                    {
                                        state.mode = ColorDialogMode::Advanced;
                                    }
                                    if
                                        ui
                                            .selectable_label(
                                                matches!(state.mode, ColorDialogMode::Web),
                                                "Web"
                                            )
                                            .clicked()
                                    {
                                        state.mode = ColorDialogMode::Web;
                                    }
                                });
                                ui.separator();

                                match state.mode {
                                    ColorDialogMode::Basic => {
                                        // Basic color grid (50 colors)
                                        let basic_colors = vec![
                                            Color32::BLACK, // 1
                                            Color32::WHITE, // 2
                                            Color32::GRAY, // 3
                                            Color32::LIGHT_GRAY, // 4
                                            Color32::DARK_GRAY, // 5
                                            Color32::RED, // 6
                                            Color32::GREEN, // 7
                                            Color32::BLUE, // 8
                                            Color32::YELLOW, // 9
                                            Color32::BROWN, // 10
                                            Color32::from_rgb(255, 105, 180), // HotPink - 11
                                            Color32::from_rgb(0, 255, 255), // Cyan - 12
                                            Color32::from_rgb(128, 0, 128), // Purple - 13
                                            Color32::from_rgb(255, 165, 0), // Orange - 14
                                            Color32::from_rgb(0, 128, 128), // Teal - 15
                                            Color32::from_rgb(255, 192, 203), // Pink - 16
                                            Color32::from_rgb(0, 100, 0), // DarkGreen - 17
                                            Color32::from_rgb(75, 0, 130), // Indigo - 18
                                            Color32::from_rgb(255, 20, 147), // DeepPink - 19
                                            Color32::from_rgb(173, 216, 230), // LightBlue - 20
                                            Color32::from_rgb(124, 252, 0), // LawnGreen - 21
                                            Color32::from_rgb(255, 250, 205), // LemonChiffon - 22
                                            Color32::from_rgb(244, 164, 96), // SandyBrown - 23
                                            Color32::from_rgb(160, 82, 45), // Sienna - 24
                                            Color32::from_rgb(95, 158, 160), // CadetBlue - 25
                                            Color32::from_rgb(123, 104, 238), // MediumSlateBlue - 26
                                            Color32::from_rgb(240, 230, 140), // Khaki - 27
                                            Color32::from_rgb(32, 178, 170), // LightSeaGreen - 28
                                            Color32::from_rgb(0, 191, 255), // DeepSkyBlue - 29
                                            Color32::from_rgb(219, 112, 147), // PaleVioletRed - 30
                                            Color32::from_rgb(238, 130, 238), // Violet - 31
                                            Color32::from_rgb(240, 128, 128), // LightCoral - 32
                                            Color32::from_rgb(255, 160, 122), // LightSalmon - 33
                                            Color32::from_rgb(250, 128, 114), // Salmon - 34
                                            Color32::from_rgb(255, 222, 173), // NavajoWhite - 35
                                            Color32::from_rgb(255, 228, 181), // Moccasin - 36
                                            Color32::from_rgb(238, 232, 170), // PaleGoldenRod - 37
                                            Color32::from_rgb(152, 251, 152), // PaleGreen - 38
                                            Color32::from_rgb(175, 238, 238), // PaleTurquoise - 39
                                            Color32::from_rgb(221, 160, 221), // Plum - 40
                                            Color32::from_rgb(255, 228, 225), // MistyRose - 41
                                            Color32::from_rgb(255, 248, 220), // Cornsilk - 42
                                            Color32::from_rgb(240, 255, 240), // Honeydew - 43
                                            Color32::from_rgb(255, 240, 245), // LavenderBlush - 44
                                            Color32::from_rgb(250, 235, 215), // AntiqueWhite - 45
                                            Color32::from_rgb(245, 245, 220), // Beige - 46
                                            Color32::from_rgb(255, 239, 213), // PapayaWhip - 47
                                            Color32::from_rgb(255, 228, 196), // Bisque - 48
                                            Color32::from_rgb(255, 235, 205), // BlanchedAlmond - 49
                                            Color32::from_rgb(0, 0, 139) // DarkBlue - 50
                                        ];

                                        ui.vertical(|ui| {
                                            for chunk in basic_colors.chunks(8) {
                                                ui.horizontal(|ui| {
                                                    for &color in chunk {
                                                        let (rect, _) = ui.allocate_exact_size(
                                                            egui::vec2(24.0, 24.0),
                                                            egui::Sense::click()
                                                        );
                                                        ui.painter().rect_filled(rect, 2.0, color);
                                                        if
                                                            ui
                                                                .interact(
                                                                    rect,
                                                                    ui.id().with(color),
                                                                    egui::Sense::click()
                                                                )
                                                                .clicked()
                                                        {
                                                            state.temp_color = color;
                                                        }
                                                    }
                                                });
                                            }
                                        });
                                    }
                                    ColorDialogMode::Web => {
                                        // Web-named colors with labels
                                        let named_colors: Vec<(&str, Color32)> = vec![
                                            ("AliceBlue", Color32::from_rgb(240, 248, 255)),
                                            ("AntiqueWhite", Color32::from_rgb(250, 235, 215)),
                                            ("Aqua", Color32::from_rgb(0, 255, 255)),
                                            ("Aquamarine", Color32::from_rgb(127, 255, 212)),
                                            ("Azure", Color32::from_rgb(240, 255, 255)),
                                            ("Beige", Color32::from_rgb(245, 245, 220)),
                                            ("Bisque", Color32::from_rgb(255, 228, 196)),
                                            ("Black", Color32::from_rgb(0, 0, 0)),
                                            ("BlanchedAlmond", Color32::from_rgb(255, 235, 205)),
                                            ("Blue", Color32::from_rgb(0, 0, 255)),
                                            ("BlueViolet", Color32::from_rgb(138, 43, 226)),
                                            ("Brown", Color32::from_rgb(165, 42, 42)),
                                            ("BurlyWood", Color32::from_rgb(222, 184, 135)),
                                            ("CadetBlue", Color32::from_rgb(95, 158, 160)),
                                            ("Chartreuse", Color32::from_rgb(127, 255, 0)),
                                            ("Chocolate", Color32::from_rgb(210, 105, 30)),
                                            ("Coral", Color32::from_rgb(255, 127, 80)),
                                            ("CornflowerBlue", Color32::from_rgb(100, 149, 237)),
                                            ("Cornsilk", Color32::from_rgb(255, 248, 220)),
                                            ("Crimson", Color32::from_rgb(220, 20, 60)),
                                            ("Cyan", Color32::from_rgb(0, 255, 255)),
                                            ("DarkBlue", Color32::from_rgb(0, 0, 139)),
                                            ("DarkCyan", Color32::from_rgb(0, 139, 139)),
                                            ("DarkGoldenRod", Color32::from_rgb(184, 134, 11)),
                                            ("DarkGray", Color32::from_rgb(169, 169, 169)),
                                            ("DarkGreen", Color32::from_rgb(0, 100, 0)),
                                            ("DarkKhaki", Color32::from_rgb(189, 183, 107)),
                                            ("DarkMagenta", Color32::from_rgb(139, 0, 139)),
                                            ("DarkOliveGreen", Color32::from_rgb(85, 107, 47)),
                                            ("DarkOrange", Color32::from_rgb(255, 140, 0)),
                                            ("DarkOrchid", Color32::from_rgb(153, 50, 204)),
                                            ("DarkRed", Color32::from_rgb(139, 0, 0)),
                                            ("DarkSalmon", Color32::from_rgb(233, 150, 122)),
                                            ("DarkSeaGreen", Color32::from_rgb(143, 188, 143)),
                                            ("DarkSlateBlue", Color32::from_rgb(72, 61, 139)),
                                            ("DarkSlateGray", Color32::from_rgb(47, 79, 79)),
                                            ("DarkTurquoise", Color32::from_rgb(0, 206, 209)),
                                            ("DarkViolet", Color32::from_rgb(148, 0, 211)),
                                            ("DeepPink", Color32::from_rgb(255, 20, 147)),
                                            ("DeepSkyBlue", Color32::from_rgb(0, 191, 255)),
                                            ("DimGray", Color32::from_rgb(105, 105, 105)),
                                            ("DodgerBlue", Color32::from_rgb(30, 144, 255)),
                                            ("FireBrick", Color32::from_rgb(178, 34, 34)),
                                            ("FloralWhite", Color32::from_rgb(255, 250, 240)),
                                            ("ForestGreen", Color32::from_rgb(34, 139, 34)),
                                            ("Fuchsia", Color32::from_rgb(255, 0, 255)),
                                            ("Gainsboro", Color32::from_rgb(220, 220, 220)),
                                            ("GhostWhite", Color32::from_rgb(248, 248, 255)),
                                            ("Gold", Color32::from_rgb(255, 215, 0)),
                                            ("GoldenRod", Color32::from_rgb(218, 165, 32)),
                                            ("Gray", Color32::from_rgb(128, 128, 128)),
                                            ("Green", Color32::from_rgb(0, 128, 0)),
                                            ("GreenYellow", Color32::from_rgb(173, 255, 47)),
                                            ("HoneyDew", Color32::from_rgb(240, 255, 240)),
                                            ("HotPink", Color32::from_rgb(255, 105, 180)),
                                            ("IndianRed", Color32::from_rgb(205, 92, 92)),
                                            ("Indigo", Color32::from_rgb(75, 0, 130)),
                                            ("Ivory", Color32::from_rgb(255, 255, 240)),
                                            ("Khaki", Color32::from_rgb(240, 230, 140)),
                                            ("Lavender", Color32::from_rgb(230, 230, 250)),
                                            ("LavenderBlush", Color32::from_rgb(255, 240, 245)),
                                            ("LawnGreen", Color32::from_rgb(124, 252, 0)),
                                            ("LemonChiffon", Color32::from_rgb(255, 250, 205)),
                                            ("LightBlue", Color32::from_rgb(173, 216, 230)),
                                            ("LightCoral", Color32::from_rgb(240, 128, 128)),
                                            ("LightCyan", Color32::from_rgb(224, 255, 255)),
                                            (
                                                "LightGoldenRodYellow",
                                                Color32::from_rgb(250, 250, 210),
                                            ),
                                            ("LightGray", Color32::from_rgb(211, 211, 211)),
                                            ("LightGreen", Color32::from_rgb(144, 238, 144)),
                                            ("LightPink", Color32::from_rgb(255, 182, 193)),
                                            ("LightSalmon", Color32::from_rgb(255, 160, 122)),
                                            ("LightSeaGreen", Color32::from_rgb(32, 178, 170)),
                                            ("LightSkyBlue", Color32::from_rgb(135, 206, 250)),
                                            ("LightSlateGray", Color32::from_rgb(119, 136, 153)),
                                            ("LightSteelBlue", Color32::from_rgb(176, 196, 222)),
                                            ("LightYellow", Color32::from_rgb(255, 255, 224)),
                                            ("Lime", Color32::from_rgb(0, 255, 0)),
                                            ("LimeGreen", Color32::from_rgb(50, 205, 50)),
                                            ("Linen", Color32::from_rgb(250, 240, 230)),
                                            ("Magenta", Color32::from_rgb(255, 0, 255)),
                                            ("Maroon", Color32::from_rgb(128, 0, 0)),
                                            ("MediumAquaMarine", Color32::from_rgb(102, 205, 170)),
                                            ("MediumBlue", Color32::from_rgb(0, 0, 205)),
                                            ("MediumOrchid", Color32::from_rgb(186, 85, 211)),
                                            ("MediumPurple", Color32::from_rgb(147, 112, 219)),
                                            ("MediumSeaGreen", Color32::from_rgb(60, 179, 113)),
                                            ("MediumSlateBlue", Color32::from_rgb(123, 104, 238)),
                                            ("MediumSpringGreen", Color32::from_rgb(0, 250, 154)),
                                            ("MediumTurquoise", Color32::from_rgb(72, 209, 204)),
                                            ("MediumVioletRed", Color32::from_rgb(199, 21, 133)),
                                            ("MidnightBlue", Color32::from_rgb(25, 25, 112)),
                                            ("MintCream", Color32::from_rgb(245, 255, 250)),
                                            ("MistyRose", Color32::from_rgb(255, 228, 225)),
                                            ("Moccasin", Color32::from_rgb(255, 228, 181)),
                                            ("NavajoWhite", Color32::from_rgb(255, 222, 173)),
                                            ("Navy", Color32::from_rgb(0, 0, 128)),
                                            ("OldLace", Color32::from_rgb(253, 245, 230)),
                                            ("Olive", Color32::from_rgb(128, 128, 0)),
                                            ("OliveDrab", Color32::from_rgb(107, 142, 35)),
                                            ("Orange", Color32::from_rgb(255, 165, 0)),
                                            ("OrangeRed", Color32::from_rgb(255, 69, 0)),
                                            ("Orchid", Color32::from_rgb(218, 112, 214))
                                        ];

                                        egui::ScrollArea
                                            ::vertical()
                                            .max_height(200.0) // Constrain height to prevent window expansion
                                            .show(ui, |ui| {
                                                for (name, color) in named_colors.iter() {
                                                    ui.horizontal(|ui| {
                                                        let (rect, _) = ui.allocate_exact_size(
                                                            egui::vec2(24.0, 24.0),
                                                            egui::Sense::click()
                                                        );
                                                        ui.painter().rect_filled(rect, 2.0, *color);
                                                        if
                                                            ui
                                                                .interact(
                                                                    rect,
                                                                    ui.id().with(name),
                                                                    egui::Sense::click()
                                                                )
                                                                .clicked() ||
                                                            ui.button(*name).clicked()
                                                        {
                                                            state.temp_color = *color;
                                                        }
                                                    });
                                                }
                                            });
                                    }
                                    ColorDialogMode::Advanced => {
                                        // Advanced RGB slider
                                        let mut rgb = [
                                            (state.temp_color.r() as f32) / 255.0,
                                            (state.temp_color.g() as f32) / 255.0,
                                            (state.temp_color.b() as f32) / 255.0,
                                        ];
                                        ui.color_edit_button_rgb(&mut rgb);
                                        state.temp_color = Color32::from_rgb(
                                            (rgb[0] * 255.0) as u8,
                                            (rgb[1] * 255.0) as u8,
                                            (rgb[2] * 255.0) as u8
                                        );
                                    }
                                }

                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label("Current:");
                                    egui::Frame
                                        ::none()
                                        .fill(state.selected_color)
                                        .inner_margin(egui::vec2(4.0, 4.0))
                                        .show(ui, |ui| {
                                            ui.allocate_space(egui::vec2(50.0, 20.0));
                                        });
                                    ui.label("Preview:");
                                    egui::Frame
                                        ::none()
                                        .fill(state.temp_color)
                                        .inner_margin(egui::vec2(4.0, 4.0))
                                        .show(ui, |ui| {
                                            ui.allocate_space(egui::vec2(50.0, 20.0));
                                        });
                                });

                                ui.separator();
                                ui.horizontal(|ui| {
                                    if ui.button("Cancel").clicked() {
                                        state.temp_color = state.selected_color; // Revert to original
                                        close_request.set(true);
                                    }
                                    if ui.button("OK").clicked() {
                                        state.selected_color = state.temp_color; // Apply the preview color
                                        close_request.set(true);
                                    }
                                });
                            });
                    }
                    if close_request.get() {
                        is_open_local = false;
                    }
                    state.is_open = is_open_local;
                }
            }
            "shape" => {
                painter.rect_filled(rect, 0.0, control.backcolor);

                let mut turtle_states = DRAWY_STATES.write().unwrap();
                if let Some(state) = turtle_states.get_mut(control_id) {
                    // Update animation
                    if !state.pending_moves.is_empty() && state.speed > 0.0 {
                        let time_delta = (current_time - state.last_update) as f32;
                        let speed_factor = (11.0 - state.speed) * 0.1;
                        state.animation_progress += time_delta / speed_factor;

                        if state.animation_progress >= 1.0 {
                            if let Some((start, end, stroke)) = state.pending_moves.pop_front() {
                                state.path.push_back((start, end, stroke));
                                state.animation_progress = 0.0;
                            }
                        }
                    }
                    state.last_update = current_time;

                    // Draw filled shape
                    if !state.fill_path.is_empty() && !state.filling {
                        painter.add(
                            egui::Shape::convex_polygon(
                                state.fill_path.clone(),
                                state.fill_color,
                                Stroke::NONE
                            )
                        );
                    }

                    // Draw completed path
                    for &(start, end, stroke) in &state.path {
                        match control.border_style.as_str() {
                            "solid" => {
                                painter.line_segment([start, end], stroke);
                            }
                            "dotted" => {
                                let dist = (end - start).length();
                                let dir = (end - start).normalized();
                                let step = 4.0;
                                let mut pos = start;
                                let mut drawn = 0.0;
                                while drawn + step < dist {
                                    let next_pos = pos + dir * step;
                                    painter.line_segment([pos, next_pos], stroke);
                                    pos = next_pos + dir * step;
                                    drawn += step * 2.0;
                                }
                            }
                            "dashed" => {
                                let dist = (end - start).length();
                                let dir = (end - start).normalized();
                                let step = 8.0;
                                let mut pos = start;
                                let mut drawn = 0.0;
                                while drawn + step < dist {
                                    let next_pos = pos + dir * step;
                                    painter.line_segment([pos, next_pos], stroke);
                                    pos = next_pos + dir * step;
                                    drawn += step * 2.0;
                                }
                            }
                            _ => {
                                painter.line_segment([start, end], stroke);
                            }
                        }
                    }

                    // Draw current animated segment
                    if let Some(&(start, end, stroke)) = state.pending_moves.front() {
                        let t = state.animation_progress.min(1.0);
                        let current_pos = start + (end - start) * t;
                        match control.border_style.as_str() {
                            "solid" => {
                                painter.line_segment([start, current_pos], stroke);
                            }
                            "dotted" => {
                                let dist = (current_pos - start).length();
                                let dir = (current_pos - start).normalized();
                                let step = 4.0;
                                let mut pos = start;
                                let mut drawn = 0.0;
                                while drawn + step < dist {
                                    let next_pos = pos + dir * step;
                                    painter.line_segment([pos, next_pos], stroke);
                                    pos = next_pos + dir * step;
                                    drawn += step * 2.0;
                                }
                            }
                            "dashed" => {
                                let dist = (current_pos - start).length();
                                let dir = (current_pos - start).normalized();
                                let step = 8.0;
                                let mut pos = start;
                                let mut drawn = 0.0;
                                while drawn + step < dist {
                                    let next_pos = pos + dir * step;
                                    painter.line_segment([pos, next_pos], stroke);
                                    pos = next_pos + dir * step;
                                    drawn += step * 2.0;
                                }
                            }
                            _ => {
                                painter.line_segment([start, current_pos], stroke);
                            }
                        }
                    }
                }
            }
            "separator" => {
                let stroke = Stroke::new(1.0, control.backcolor);
                if control.orientation == "horizontal" {
                    let y = rect.min.y + rect.height() / 2.0;
                    ui.painter().line_segment(
                        [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                        stroke
                    );
                } else {
                    let x = rect.min.x + rect.width() / 2.0;
                    ui.painter().line_segment(
                        [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                        stroke
                    );
                }
            }
            "gridlayout" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut grid_states = GRIDLAYOUT_STATES.write().unwrap();
                if let Some(state) = grid_states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(rect, |ui| {
                        let total_spacing_width = if state.cols > 1 {
                            ((state.cols - 1) as f32) * state.spacing
                        } else {
                            0.0
                        };
                        let cell_width = (rect.width() - total_spacing_width) / (state.cols as f32);
                        let total_spacing_height = if state.rows > 1 {
                            ((state.rows - 1) as f32) * state.spacing
                        } else {
                            0.0
                        };
                        let cell_height =
                            (rect.height() - total_spacing_height) / (state.rows as f32);

                        for row in 0..state.rows {
                            for col in 0..state.cols {
                                if
                                    let Some(Some(child_id)) = state.cell_controls
                                        .get(row)
                                        .and_then(|r| r.get(col))
                                {
                                    if let Some(child) = controls.get(child_id) {
                                        let child_pos = Pos2::new(
                                            rect.min.x +
                                                (col as f32) * (cell_width + state.spacing),
                                            rect.min.y +
                                                (row as f32) * (cell_height + state.spacing)
                                        );
                                        let child_settings = ControlSettings {
                                            position: child_pos,
                                            width: child.width, // Preserve original width
                                            height: child.height, // Preserve original height
                                            ..child.clone()
                                        };
                                        render_control(
                                            ui,
                                            child_id,
                                            &child_settings,
                                            ctx,
                                            textbox_texts
                                        );
                                    }
                                }
                            }
                        }
                        // Grid lines rendering remains unchanged
                        if state.show_grid_lines {
                            let stroke = Stroke::new(state.line_thickness, state.line_color);
                            for row in 0..=state.rows {
                                let y = rect.min.y + (row as f32) * (cell_height + state.spacing);
                                let end_x = if row < state.rows {
                                    rect.max.x
                                } else {
                                    rect.min.x +
                                        (state.cols as f32) * (cell_width + state.spacing) -
                                        state.spacing
                                };
                                ui.painter().line_segment(
                                    [Pos2::new(rect.min.x, y), Pos2::new(end_x, y)],
                                    stroke
                                );
                            }
                            for col in 0..=state.cols {
                                let x = rect.min.x + (col as f32) * (cell_width + state.spacing);
                                let end_y = if col < state.cols {
                                    rect.max.y
                                } else {
                                    rect.min.y +
                                        (state.rows as f32) * (cell_height + state.spacing) -
                                        state.spacing
                                };
                                ui.painter().line_segment(
                                    [Pos2::new(x, rect.min.y), Pos2::new(x, end_y)],
                                    stroke
                                );
                            }
                        }
                    });
                }
            }
            "horizontallayout" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut hlayout_states = HORIZONTALLAYOUT_STATES.write().unwrap();
                if let Some(state) = hlayout_states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(rect, |ui| {
                        let mut x_offset = rect.min.x;
                        for child_id in &control.children {
                            if let Some(child) = controls.get(child_id) {
                                let child_pos = Pos2::new(x_offset, rect.min.y);
                                let child_settings = ControlSettings {
                                    position: child_pos,
                                    ..child.clone()
                                };
                                render_control(ui, child_id, &child_settings, ctx, textbox_texts);
                                x_offset += child.width + state.spacing;
                            }
                        }
                    });
                }
            }
            "verticallayout" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut vlayout_states = VERTICALLAYOUT_STATES.write().unwrap();
                if let Some(state) = vlayout_states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(rect, |ui| {
                        let mut y_offset = rect.min.y;
                        for child_id in &control.children {
                            if let Some(child) = controls.get(child_id) {
                                let child_pos = Pos2::new(rect.min.x, y_offset);
                                let child_settings = ControlSettings {
                                    position: child_pos,
                                    ..child.clone()
                                };
                                render_control(ui, child_id, &child_settings, ctx, textbox_texts);
                                y_offset += child.height + state.spacing;
                            }
                        }
                    });
                }
            }
            "flowlayout" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut flow_states = FLOWLAYOUT_STATES.write().unwrap();
                if let Some(state) = flow_states.get_mut(control_id) {
                    egui::ScrollArea
                        ::vertical()
                        .auto_shrink([false, false]) // Prevents shrinking smaller than content
                        .show(ui, |ui| {
                            let mut x_offset = rect.min.x;
                            let mut y_offset = rect.min.y;
                            let max_width = rect.width();

                            for child_id in &control.children {
                                if let Some(child) = controls.get(child_id) {
                                    // Wrap to next line if exceeding width
                                    if state.wrap && x_offset + child.width > rect.max.x {
                                        x_offset = rect.min.x;
                                        y_offset += child.height + state.spacing;
                                    }
                                    let child_pos = Pos2::new(x_offset, y_offset);
                                    let child_settings = ControlSettings {
                                        position: child_pos,
                                        ..child.clone()
                                    };
                                    render_control(
                                        ui,
                                        child_id,
                                        &child_settings,
                                        ctx,
                                        textbox_texts
                                    );
                                    x_offset += child.width + state.spacing;
                                }
                            }
                        });
                }
            }
            "datetimepicker" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut states = DATETIMEPICKER_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    // Render the main button that shows the current selection.
                    ui.allocate_ui_at_rect(rect, |ui| {
                        if
                            ui
                                .add_sized(
                                    rect.size(),
                                    egui::Button::new(&control.text).fill(control.backcolor)
                                )
                                .clicked()
                        {
                            state.is_open = !state.is_open;
                        }
                    });

                    // When state.is_open is true, render the date picker in a floating area.
                    if state.is_open {
                        // Calculate the popup position (bottom-left of the button)
                        let popup_pos = egui::pos2(rect.min.x, rect.max.y);
                        let popup_id = Id::new(format!("datetimepicker_popup_{}", control_id));

                        egui::Area
                            ::new(popup_id)
                            .order(egui::Order::Foreground)
                            .fixed_pos(popup_pos)
                            .show(ui.ctx(), |ui| {
                                // Wrap the content in a popup frame with white background and shadow.
                                egui::Frame
                                    ::popup(ui.style())
                                    .fill(egui::Color32::WHITE)
                                    .outer_margin(egui::vec2(4.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                // Left arrow to decrement month.
                                                if ui.button("◀").clicked() {
                                                    state.selected_datetime =
                                                        state.selected_datetime
                                                            .checked_sub_months(
                                                                chrono::Months::new(1)
                                                            )
                                                            .unwrap();
                                                }

                                                // Render dropdowns for Month and Year.
                                                ui.horizontal(|ui| {
                                                    // Month ComboBox
                                                    egui::ComboBox
                                                        ::from_label("Month")
                                                        .selected_text(
                                                            state.selected_datetime
                                                                .format("%B")
                                                                .to_string()
                                                        )
                                                        .show_ui(ui, |ui| {
                                                            for month in 1..=12 {
                                                                let month_name = chrono::NaiveDate
                                                                    ::from_ymd_opt(2023, month, 1)
                                                                    .unwrap()
                                                                    .format("%B")
                                                                    .to_string();
                                                                if
                                                                    ui
                                                                        .selectable_label(
                                                                            state.selected_datetime.month() ==
                                                                                month,
                                                                            month_name
                                                                        )
                                                                        .clicked()
                                                                {
                                                                    state.selected_datetime =
                                                                        state.selected_datetime
                                                                            .with_month(month)
                                                                            .unwrap();
                                                                }
                                                            }
                                                        });

                                                    // Year ComboBox: display years 1900 to 2100
                                                    egui::ComboBox
                                                        ::from_label("Year")
                                                        .selected_text(
                                                            state.selected_datetime
                                                                .year()
                                                                .to_string()
                                                        )
                                                        .show_ui(ui, |ui| {
                                                            for year in 1900..=2100 {
                                                                if
                                                                    ui
                                                                        .selectable_label(
                                                                            state.selected_datetime.year() ==
                                                                                year,
                                                                            year.to_string()
                                                                        )
                                                                        .clicked()
                                                                {
                                                                    state.selected_datetime =
                                                                        state.selected_datetime
                                                                            .with_year(year)
                                                                            .unwrap();
                                                                }
                                                            }
                                                        });
                                                });

                                                // Right arrow to increment month.
                                                if ui.button("▶").clicked() {
                                                    state.selected_datetime =
                                                        state.selected_datetime
                                                            .checked_add_months(
                                                                chrono::Months::new(1)
                                                            )
                                                            .unwrap();
                                                }
                                            });

                                            ui.separator();

                                            // Render weekday headers.
                                            let weekdays = [
                                                "Mon",
                                                "Tue",
                                                "Wed",
                                                "Thu",
                                                "Fri",
                                                "Sat",
                                                "Sun",
                                            ];
                                            ui.horizontal(|ui| {
                                                for day in weekdays.iter() {
                                                    ui.add_sized(
                                                        [40.0, 20.0],
                                                        egui::Label::new(*day)
                                                    );
                                                }
                                            });

                                            ui.separator();

                                            // Render grid for days.
                                            let first_day = chrono::NaiveDate
                                                ::from_ymd_opt(
                                                    state.selected_datetime.year(),
                                                    state.selected_datetime.month(),
                                                    1
                                                )
                                                .unwrap();
                                            let start_day = first_day
                                                .weekday()
                                                .num_days_from_monday() as usize;
                                            let days_in_month = first_day
                                                .checked_add_months(chrono::Months::new(1))
                                                .unwrap()
                                                .pred_opt()
                                                .unwrap()
                                                .day();

                                            let mut day_grid: Vec<Option<u32>> =
                                                vec![None; start_day];
                                            for day in 1..=days_in_month {
                                                day_grid.push(Some(day));
                                            }
                                            while day_grid.len() % 7 != 0 {
                                                day_grid.push(None);
                                            }

                                            for row in day_grid.chunks(7) {
                                                ui.horizontal(|ui| {
                                                    for cell in row {
                                                        if let Some(d) = cell {
                                                            if
                                                                ui
                                                                    .add_sized(
                                                                        [40.0, 30.0],
                                                                        egui::Button::new(
                                                                            d.to_string()
                                                                        )
                                                                    )
                                                                    .clicked()
                                                            {
                                                                state.selected_datetime =
                                                                    state.selected_datetime
                                                                        .with_day(*d)
                                                                        .unwrap();
                                                            }
                                                        } else {
                                                            ui.add_sized(
                                                                [40.0, 30.0],
                                                                egui::Label::new(" ")
                                                            );
                                                        }
                                                    }
                                                });
                                            }

                                            ui.separator();

                                            // OK button to confirm the selection and close the date picker.
                                            if ui.button("OK").clicked() {
                                                let mut controls = CONTROLS.write().unwrap();
                                                if let Some(ctrl) = controls.get_mut(control_id) {
                                                    ctrl.text = state.selected_datetime
                                                        .format("%Y-%m-%d")
                                                        .to_string();
                                                }
                                                state.is_open = false;
                                            }
                                        });
                                    });
                            });
                    }
                }
            }
            "timerpicker" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut states = TIMERPICKER_STATES.lock().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    // Render the main button showing the current time
                    ui.allocate_ui_at_rect(rect, |ui| {
                        if
                            ui
                                .add_sized(
                                    rect.size(),
                                    egui::Button::new(&control.text).fill(control.backcolor)
                                )
                                .clicked()
                        {
                            state.is_open = !state.is_open;
                        }
                    });

                    // Render the popup when open
                    if state.is_open {
                        let popup_pos = egui::pos2(rect.min.x, rect.max.y);
                        let popup_id = Id::new(format!("timerpicker_popup_{}", control_id));

                        egui::Area
                            ::new(popup_id)
                            .order(egui::Order::Foreground)
                            .fixed_pos(popup_pos)
                            .show(ui.ctx(), |ui| {
                                egui::Frame
                                    ::popup(ui.style())
                                    .fill(egui::Color32::WHITE)
                                    .outer_margin(egui::vec2(4.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                // Hours ComboBox (0-23)
                                                egui::ComboBox
                                                    ::from_label("Hours")
                                                    .selected_text(
                                                        format!("{:02}", state.selected_time.hour())
                                                    )
                                                    .show_ui(ui, |ui| {
                                                        for hour in 0..24 {
                                                            if
                                                                ui
                                                                    .selectable_label(
                                                                        state.selected_time.hour() ==
                                                                            hour,
                                                                        format!("{:02}", hour)
                                                                    )
                                                                    .clicked()
                                                            {
                                                                state.selected_time =
                                                                    state.selected_time
                                                                        .with_hour(hour)
                                                                        .unwrap();
                                                            }
                                                        }
                                                    });

                                                // Minutes ComboBox (0-59)
                                                egui::ComboBox
                                                    ::from_label("Minutes")
                                                    .selected_text(
                                                        format!(
                                                            "{:02}",
                                                            state.selected_time.minute()
                                                        )
                                                    )
                                                    .show_ui(ui, |ui| {
                                                        for minute in 0..60 {
                                                            if
                                                                ui
                                                                    .selectable_label(
                                                                        state.selected_time.minute() ==
                                                                            minute,
                                                                        format!("{:02}", minute)
                                                                    )
                                                                    .clicked()
                                                            {
                                                                state.selected_time =
                                                                    state.selected_time
                                                                        .with_minute(minute)
                                                                        .unwrap();
                                                            }
                                                        }
                                                    });

                                                // Seconds ComboBox (0-59), only if format includes seconds
                                                if state.format.contains("%S") {
                                                    egui::ComboBox
                                                        ::from_label("Seconds")
                                                        .selected_text(
                                                            format!(
                                                                "{:02}",
                                                                state.selected_time.second()
                                                            )
                                                        )
                                                        .show_ui(ui, |ui| {
                                                            for second in 0..60 {
                                                                if
                                                                    ui
                                                                        .selectable_label(
                                                                            state.selected_time.second() ==
                                                                                second,
                                                                            format!("{:02}", second)
                                                                        )
                                                                        .clicked()
                                                                {
                                                                    state.selected_time =
                                                                        state.selected_time
                                                                            .with_second(second)
                                                                            .unwrap();
                                                                }
                                                            }
                                                        });
                                                }
                                            });

                                            ui.separator();

                                            // OK button to confirm and close
                                            if ui.button("OK").clicked() {
                                                let mut controls = CONTROLS.write().unwrap();
                                                if let Some(ctrl) = controls.get_mut(control_id) {
                                                    ctrl.text = state.selected_time
                                                        .format(&state.format)
                                                        .to_string();
                                                }
                                                state.is_open = false;
                                            }
                                        });
                                    });
                            });
                    }
                }
            }

            "table" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);

                let inner_rect = Rect {
                    min: rect.min + Vec2::new(control.padding.0, control.padding.1),
                    max: rect.max - Vec2::new(control.padding.2, control.padding.3),
                };

                let mut states = TABLE_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    ui.allocate_ui_at_rect(inner_rect, |ui| {
                        // Customize the UI style for the table
                        let mut style = ui.style_mut();
                        style.visuals.widgets.inactive.bg_fill = control.backcolor;
                        style.visuals.widgets.hovered.bg_fill = lighten(control.backcolor, 0.1);
                        style.visuals.widgets.active.bg_fill = lighten(control.backcolor, 0.2);
                        style.visuals.widgets.inactive.fg_stroke = Stroke::new(
                            1.0,
                            control.forecolor
                        );
                        style.visuals.widgets.hovered.fg_stroke = Stroke::new(
                            1.0,
                            control.forecolor
                        );
                        style.visuals.widgets.active.fg_stroke = Stroke::new(
                            1.0,
                            control.forecolor
                        );
                        style.text_styles.insert(
                            egui::TextStyle::Body, // For table cells
                            FontId::new(control.fontsize, get_font_family(&control.fontname))
                        );
                        style.text_styles.insert(
                            egui::TextStyle::Button, // For header buttons
                            FontId::new(control.fontsize, get_font_family(&control.fontname))
                        );

                        let table = TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(
                                egui::Layout
                                    ::default()
                                    .with_main_align(control.text_alignment[0]) // Horizontal alignment
                                    .with_cross_align(control.text_alignment[1])
                            ) // Vertical alignment
                            .columns(Column::auto(), state.headers.len());

                        table
                            .header(20.0, |mut header| {
                                for (i, h) in state.headers.iter().enumerate() {
                                    header.col(|ui| {
                                        ui.with_layout(
                                            egui::Layout
                                                ::default()
                                                .with_main_align(control.text_alignment[0])
                                                .with_cross_align(control.text_alignment[1]),
                                            |ui| {
                                                if ui.button(h).clicked() {
                                                    if state.sort_column == Some(i) {
                                                        state.sort_ascending =
                                                            !state.sort_ascending;
                                                    } else {
                                                        state.sort_column = Some(i);
                                                        state.sort_ascending = true;
                                                    }
                                                    if let Some(col) = state.sort_column {
                                                        if col < state.headers.len() {
                                                            state.rows.sort_by(|a, b| {
                                                                let binding = String::new();
                                                                let a_val = a
                                                                    .get(col)
                                                                    .unwrap_or(&binding);
                                                                let binding = String::new();
                                                                let b_val = b
                                                                    .get(col)
                                                                    .unwrap_or(&binding);
                                                                if state.sort_ascending {
                                                                    a_val.cmp(b_val)
                                                                } else {
                                                                    b_val.cmp(a_val)
                                                                }
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        );
                                    });
                                }
                            })
                            .body(|mut body| {
                                for row_data in &state.rows {
                                    body.row(state.row_height, |mut row| {
                                        for cell in row_data {
                                            row.col(|ui| {
                                                ui.with_layout(
                                                    egui::Layout
                                                        ::default()
                                                        .with_main_align(control.text_alignment[0])
                                                        .with_cross_align(
                                                            control.text_alignment[1]
                                                        ),
                                                    |ui| {
                                                        ui.label(cell);
                                                    }
                                                );
                                            });
                                        }
                                    });
                                }
                            });
                    });
                }
            }
            "menu" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);

                let inner_rect = Rect {
                    min: rect.min + Vec2::new(control.padding.0, control.padding.1),
                    max: rect.max - Vec2::new(control.padding.2, control.padding.3),
                };

                let mut states = match MENU_STATES.write() {
                    Ok(states) => states,
                    Err(e) => {
                        eprintln!("Failed to lock MENU_STATES: {}", e);
                        return;
                    }
                };

                if let Some(state) = states.get_mut(control_id) {
                    let items = state.items.clone();

                    ui.allocate_ui_at_rect(inner_rect, |ui| {
                        let mut style = ui.style_mut();
                        style.visuals.widgets.inactive.bg_fill = control.backcolor;
                        style.visuals.widgets.hovered.bg_fill = lighten(control.backcolor, 0.1); // 10% lighter
                        style.visuals.widgets.active.bg_fill = lighten(control.backcolor, 0.2); // 20% lighter
                        style.visuals.widgets.inactive.fg_stroke = Stroke::new(
                            1.0,
                            control.forecolor
                        );
                        style.visuals.widgets.hovered.fg_stroke = Stroke::new(
                            1.0,
                            control.forecolor
                        );
                        style.visuals.widgets.active.fg_stroke = Stroke::new(
                            1.0,
                            control.forecolor
                        );
                        style.text_styles.insert(
                            egui::TextStyle::Button,
                            FontId::new(control.fontsize, get_font_family(&control.fontname))
                        );

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add_space(10.0); // Left margin
                            for item in items {
                                ui.menu_button(&item.label, |ui| {
                                    let child_items = item.children.clone();
                                    render_menu_items(ui, &child_items, state, ctx, &control);
                                });
                            }
                        });
                    });
                } else {
                    eprintln!("Menu state not found for ID: {}", control_id);
                }
            }
            "timer" => {}
            "pages" => {
                let rect = Rect::from_min_size(pos, size);
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                if control.border && !control.use_as_default_panel {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::GRAY),
                        StrokeKind::Outside
                    );
                }

                let mut pages_states = PAGES_STATES.write().expect("Failed to lock PAGES_STATES");
                if let Some(state) = pages_states.get_mut(control_id) {
                    // Set tab height to 0 when tabs are hidden, 30 when shown
                    let tab_height = if control.use_as_default_panel { 0.0 } else { 30.0 };
                    let content_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x, rect.min.y + tab_height),
                        Vec2::new(rect.width(), rect.height() - tab_height)
                    );

                    // Render tab buttons only if use_as_default_panel is false
                    if !control.use_as_default_panel {
                        ui.allocate_ui_at_rect(
                            Rect::from_min_size(rect.min, Vec2::new(rect.width(), tab_height)),
                            |ui| {
                                ui.horizontal(|ui| {
                                    for (i, page) in state.pages.iter().enumerate() {
                                        let tab_response = ui.button(&page.title);
                                        if tab_response.clicked() && state.active_page_index != i {
                                            state.active_page_index = i;
                                        }
                                        if i == state.active_page_index {
                                            ui.painter().rect_filled(
                                                tab_response.rect,
                                                0.0,
                                                Color32::from_gray(200)
                                            );
                                            ui.painter().text(
                                                tab_response.rect.center(),
                                                Align2::CENTER_CENTER,
                                                &page.title,
                                                FontId::new(14.0, FontFamily::Proportional),
                                                Color32::BLACK
                                            );
                                        }
                                    }
                                });
                            }
                        );
                    }

                    // Render the active page’s content
                    let controls = CONTROLS.read().expect("Failed to lock CONTROLS");
                    ui.allocate_ui_at_rect(content_rect, |ui| {
                        if state.active_page_index < state.pages.len() {
                            let active_page = &state.pages[state.active_page_index];
                            for child_id in &active_page.control_ids {
                                if let Some(child) = controls.get(child_id) {
                                    let child_pos = content_rect.min + child.position.to_vec2();
                                    let mut child_settings = child.clone();
                                    child_settings.position = child_pos;
                                    render_control(
                                        ui,
                                        child_id,
                                        &child_settings,
                                        ctx,
                                        textbox_texts
                                    );
                                }
                            }
                        }
                    });
                }
            }
            "scrollbar" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut states = SCROLLBAR_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    let range = state.max - state.min;
                    let thumb_size = if state.orientation == ScrollBarOrientation::Vertical {
                        ((rect.height() * state.large_change) / range).max(20.0)
                    } else {
                        ((rect.width() * state.large_change) / range).max(20.0)
                    };
                    let progress = (state.value - state.min) / range;
                    let thumb_pos = if state.orientation == ScrollBarOrientation::Vertical {
                        rect.min.y + (rect.height() - thumb_size) * progress
                    } else {
                        rect.min.x + (rect.width() - thumb_size) * progress
                    };
                    let thumb_rect = if state.orientation == ScrollBarOrientation::Vertical {
                        Rect::from_min_size(
                            Pos2::new(rect.min.x, thumb_pos),
                            Vec2::new(rect.width(), thumb_size)
                        )
                    } else {
                        Rect::from_min_size(
                            Pos2::new(thumb_pos, rect.min.y),
                            Vec2::new(thumb_size, rect.height())
                        )
                    };
                    let response = ui.interact(thumb_rect, ui.next_auto_id(), Sense::drag());
                    ui.painter().rect_filled(thumb_rect, 0.0, Color32::DARK_GRAY);
                    if response.dragged() {
                        let delta = response.drag_delta();
                        let movement = if state.orientation == ScrollBarOrientation::Vertical {
                            (delta.y / (rect.height() - thumb_size)) * range
                        } else {
                            (delta.x / (rect.width() - thumb_size)) * range
                        };
                        state.value = (state.value + movement).clamp(state.min, state.max);
                    }
                }
            }
            "picturebox" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut states = PICTUREBOX_STATES.write().unwrap();
                if let Some(state) = states.get_mut(control_id) {
                    if let Some(path) = &state.image_path {
                        if state.texture_handle.is_none() {
                            if let Ok(image_data) = fs::read(path) {
                                if let Ok(image) = image::load_from_memory(&image_data) {
                                    let rgba = image.to_rgba8();
                                    let size = [rgba.width() as usize, rgba.height() as usize];
                                    let pixels = rgba.into_raw();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        &pixels
                                    );
                                    state.texture_handle = Some(
                                        ctx.load_texture(
                                            control_id.clone(),
                                            color_image,
                                            Default::default()
                                        )
                                    );
                                }
                            }
                        }
                        if let Some(texture) = &state.texture_handle {
                            let img_size = Vec2::new(
                                texture.size()[0] as f32,
                                texture.size()[1] as f32
                            );
                            let display_size = match state.size_mode {
                                PictureBoxSizeMode::Normal => img_size,
                                PictureBoxSizeMode::Stretch => rect.size(),
                                PictureBoxSizeMode::Zoom => {
                                    let scale = (rect.width() / img_size.x).min(
                                        rect.height() / img_size.y
                                    );
                                    Vec2::new(img_size.x * scale, img_size.y * scale)
                                }
                            };
                            let img_rect = Rect::from_center_size(rect.center(), display_size);
                            ui.painter().image(
                                texture.id(),
                                img_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE
                            );
                        }
                    }
                }
            }
            "progressbar" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let states = PROGRESSBAR_STATES.read().unwrap();
                if let Some(state) = states.get(control_id) {
                    match state.style {
                        ProgressBarStyle::Solid => {
                            let progress = (state.value - state.min) / (state.max - state.min);
                            let bar_width = rect.width() * progress.clamp(0.0, 1.0);
                            let bar_rect = Rect::from_min_size(
                                rect.min,
                                Vec2::new(bar_width, rect.height())
                            );
                            ui.painter().rect_filled(bar_rect, 0.0, state.bar_color);
                        }
                        ProgressBarStyle::Marquee => {
                            let time = ctx.input(|i| i.time);
                            let offset = (
                                (time.sin() as f32) * rect.width() * 0.5 +
                                rect.width() * 0.5
                            ).rem_euclid(rect.width());
                            let bar_width = rect.width() * 0.3;
                            let bar_rect = Rect::from_min_size(
                                Pos2::new(rect.min.x + offset, rect.min.y),
                                Vec2::new(bar_width, rect.height())
                            );
                            ui.painter().rect_filled(
                                bar_rect.intersect(rect),
                                0.0,
                                state.bar_color
                            );
                        }
                    }
                }
            }
            "listbox" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut listbox_items = LISTBOX_ITEMS.write().unwrap();
                if let Some(items_map) = listbox_items.get_mut(control_id) {
                    if let Some((items, selected_index)) = items_map.get_mut(control_id) {
                        ui.allocate_ui_at_rect(rect, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, arc) in items.iter().enumerate() {
                                    let value = arc.lock().unwrap();
                                    let item_text = value.to_string();
                                    if
                                        ui
                                            .selectable_label(
                                                i == *selected_index,
                                                item_text.clone()
                                            )
                                            .clicked()
                                    {
                                        *selected_index = i;
                                        let mut controls = CONTROLS.write().unwrap();
                                        if let Some(ctrl) = controls.get_mut(control_id) {
                                            ctrl.text = item_text;
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            }
            "combobox" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let mut combobox_items = COMBOBOX_ITEMS.write().unwrap();
                if let Some(items_map) = combobox_items.get_mut(control_id) {
                    if let Some((items, selected_index)) = items_map.get_mut(control_id) {
                        let selected_text = items
                            .get(*selected_index)
                            .map_or("".to_string(), |arc| { arc.lock().unwrap().to_string() });
                        ui.allocate_ui_at_rect(rect, |ui| {
                            egui::ComboBox
                                ::from_id_source(control_id)
                                .selected_text(selected_text.clone())
                                .show_ui(ui, |ui| {
                                    for (i, arc) in items.iter().enumerate() {
                                        let value = arc.lock().unwrap();
                                        let item_text = value.to_string();
                                        if
                                            ui
                                                .selectable_value(
                                                    selected_index,
                                                    i,
                                                    item_text.clone()
                                                )
                                                .clicked()
                                        {
                                            *selected_index = i;
                                            let mut controls = CONTROLS.write().unwrap();
                                            if let Some(ctrl) = controls.get_mut(control_id) {
                                                ctrl.text = item_text;
                                            }
                                        }
                                    }
                                });
                            ui.end_row();
                        });
                    }
                }
            }
            "groupbox" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                if control.border {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::GRAY),
                        StrokeKind::Outside
                    );
                    let title_pos = pos + Vec2::new(15.0, -2.0);
                    ui.painter().text(
                        title_pos,
                        Align2::LEFT_CENTER,
                        &control.text,
                        FontId::new(
                            control.fontsize,
                            FontFamily::Name(control.fontname.clone().into())
                        ),
                        control.forecolor
                    );
                }
                // Render children relative to parent position
                let controls = CONTROLS.read().unwrap();
                ui.allocate_ui_at_rect(rect, |ui| {
                    for child_id in &control.children {
                        if let Some(child) = controls.get(child_id) {
                            let child_pos = rect.min + child.position.to_vec2();
                            let child_settings = ControlSettings {
                                position: child_pos,
                                ..child.clone()
                            };
                            render_control(ui, child_id, &child_settings, ctx, textbox_texts);
                        }
                    }
                });
            }
            "panel" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                if control.border {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::GRAY),
                        StrokeKind::Inside
                    );
                }
                // Render children relative to parent position
                let controls = CONTROLS.read().unwrap();
                ui.allocate_ui_at_rect(rect, |ui| {
                    for child_id in &control.children {
                        if let Some(child) = controls.get(child_id) {
                            let child_pos = rect.min + child.position.to_vec2();
                            let child_settings = ControlSettings {
                                position: child_pos,
                                ..child.clone()
                            };
                            render_control(ui, child_id, &child_settings, ctx, textbox_texts);
                        }
                    }
                });
            }
            "card" => {
                if control.shadow {
                    ui.painter().rect_filled(rect.expand(4.0), 4.0, Color32::from_black_alpha(50));
                }
                ui.painter().rect_filled(rect, 4.0, control.backcolor);
                // Render children relative to parent position with padding
                let controls = CONTROLS.read().unwrap();
                ui.allocate_ui_at_rect(rect, |ui| {
                    for child_id in &control.children {
                        if let Some(child) = controls.get(child_id) {
                            let child_pos =
                                rect.min +
                                Vec2::new(control.padding.0, control.padding.1) +
                                child.position.to_vec2();
                            let child_settings = ControlSettings {
                                position: child_pos,
                                ..child.clone()
                            };
                            render_control(ui, child_id, &child_settings, ctx, textbox_texts);
                        }
                    }
                });
            }
            "sidebar" => {
                let form_rect = ui.clip_rect();
                let height = form_rect.height();
                let sidebar_rect = match control.dock {
                    DockStyle::Left =>
                        Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(control.width, height)),
                    DockStyle::Right =>
                        Rect::from_min_size(
                            Pos2::new(form_rect.width() - control.width, 0.0),
                            Vec2::new(control.width, height)
                        ),
                    _ => rect, // Use computed rect if no docking
                };
                ui.painter().rect_filled(sidebar_rect, 0.0, control.backcolor);
                // Render children relative to parent position
                let controls = CONTROLS.read().unwrap();
                ui.allocate_ui_at_rect(sidebar_rect, |ui| {
                    for child_id in &control.children {
                        if let Some(child) = controls.get(child_id) {
                            let child_pos = sidebar_rect.min + child.position.to_vec2();
                            let child_settings = ControlSettings {
                                position: child_pos,
                                ..child.clone()
                            };
                            render_control(ui, child_id, &child_settings, ctx, textbox_texts);
                        }
                    }
                });
            }
            "checkbox" => {
                let mut checked = control.checked;
                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.add(egui::widgets::Checkbox::new(&mut checked, &control.text));
                });
                if ui.input(|i| i.pointer.any_released()) {
                    let mut controls = CONTROLS.write().unwrap();
                    if let Some(ctrl) = controls.get_mut(control_id) {
                        ctrl.checked = checked;
                        if let Some(callback) = &ctrl.callback {
                            match callback {
                                Value::Function { body, closure, object, .. } => {
                                    let mut interpreter = GLOBAL_INTERPRETER.lock().unwrap();
                                    let mut local_env = Arc::new(
                                        Mutex::new(Environment::new(Some(closure.clone())))
                                    );
                                    if let Some(obj) = object {
                                        let value = obj.lock().unwrap().clone();
                                        local_env.lock().unwrap().define("this".to_string(), value);
                                    }
                                    let _ = interpreter.visit_block(body, &mut local_env);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            "radiobox" => {
                if let Some(group) = &control.group {
                    let selected_id = SELECTED_RADIOBOXES.read()
                        .unwrap()
                        .get(&(control.form_id.clone(), group.clone()))
                        .cloned();
                    let is_selected = selected_id.as_ref().map_or(false, |id| id == control_id);
                    let response = ui.allocate_ui_at_rect(rect, |ui| {
                        ui.add(egui::widgets::RadioButton::new(is_selected, &control.text))
                    });
                    if response.inner.clicked() && !is_selected {
                        let mut selected_radioboxes = SELECTED_RADIOBOXES.write().unwrap();
                        selected_radioboxes.insert(
                            (control.form_id.clone(), group.clone()),
                            control_id.clone()
                        );
                        let mut controls = CONTROLS.write().unwrap();
                        for (id, ctrl) in controls.iter_mut() {
                            if
                                ctrl.form_id == control.form_id &&
                                ctrl.group.as_ref() == Some(group)
                            {
                                ctrl.checked = id == control_id;
                            }
                        }
                        if let Some(callback) = &control.callback {
                            match callback {
                                Value::Function { body, closure, object, .. } => {
                                    let mut interpreter = GLOBAL_INTERPRETER.lock().unwrap();
                                    let mut local_env = Arc::new(
                                        Mutex::new(Environment::new(Some(closure.clone())))
                                    );
                                    if let Some(obj) = object {
                                        let value = obj.lock().unwrap().clone();
                                        local_env.lock().unwrap().define("this".to_string(), value);
                                    }
                                    let _ = interpreter.visit_block(body, &mut local_env);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            "label" => {
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let inner_rect = Rect {
                    min: rect.min + Vec2::new(control.padding.0, control.padding.1),
                    max: rect.max - Vec2::new(control.padding.2, control.padding.3),
                };
                let font_id = FontId::new(control.fontsize, get_font_family(&control.fontname));
                let mut job = LayoutJob::default();
                job.append(&control.text, 0.0, TextFormat {
                    font_id,
                    color: control.forecolor,
                    ..Default::default()
                });
                let galley = ui.fonts(|f| f.layout_job(job));
                let text_size = galley.rect.size();
                let align_x = control.text_alignment[0];
                let align_y = control.text_alignment[1];
                let offset_x = match align_x {
                    egui::Align::Min => 0.0,
                    egui::Align::Center => (inner_rect.width() - text_size.x) / 2.0,
                    egui::Align::Max => inner_rect.width() - text_size.x,
                };
                let offset_y = match align_y {
                    egui::Align::Min => 0.0,
                    egui::Align::Center => (inner_rect.height() - text_size.y) / 2.0,
                    egui::Align::Max => inner_rect.height() - text_size.y,
                };
                let text_pos = inner_rect.min + Vec2::new(offset_x, offset_y);
                ui.painter().galley(text_pos, galley, control.forecolor);
            }
            "button" => {
                let mut bg_color = control.backcolor;
                let response = ui.interact(rect, ui.next_auto_id(), Sense::click());
                if response.hovered() {
                    bg_color = Color32::from_rgb(
                        ((bg_color.r() as f32) * 1.1).min(255.0) as u8,
                        ((bg_color.g() as f32) * 1.1).min(255.0) as u8,
                        ((bg_color.b() as f32) * 1.1).min(255.0) as u8
                    );
                }
                if response.clicked() {
                    bg_color = Color32::from_rgb(
                        ((bg_color.r() as f32) * 0.9).max(0.0) as u8,
                        ((bg_color.g() as f32) * 0.9).max(0.0) as u8,
                        ((bg_color.b() as f32) * 0.9).max(0.0) as u8
                    );
                }
                ui.painter().rect_filled(rect, 0.0, bg_color);
                let inner_rect = Rect {
                    min: rect.min + Vec2::new(control.padding.0, control.padding.1),
                    max: rect.max - Vec2::new(control.padding.2, control.padding.3),
                };
                let font_id = FontId::new(control.fontsize, get_font_family(&control.fontname));
                let mut job = LayoutJob::default();
                job.append(&control.text, 0.0, TextFormat {
                    font_id,
                    color: control.forecolor,
                    ..Default::default()
                });
                let galley = ui.fonts(|f| f.layout_job(job));
                let text_size = galley.rect.size();
                let align_x = control.text_alignment[0];
                let align_y = control.text_alignment[1];
                let offset_x = match align_x {
                    egui::Align::Min => 0.0,
                    egui::Align::Center => (inner_rect.width() - text_size.x) / 2.0,
                    egui::Align::Max => inner_rect.width() - text_size.x,
                };
                let offset_y = match align_y {
                    egui::Align::Min => 0.0,
                    egui::Align::Center => (inner_rect.height() - text_size.y) / 2.0,
                    egui::Align::Max => inner_rect.height() - text_size.y,
                };
                let text_pos = inner_rect.min + Vec2::new(offset_x, offset_y);
                ui.painter().galley(text_pos, galley, control.forecolor);
                if response.clicked() {
                    if let Some(callback) = &control.callback {
                        match callback {
                            Value::Function { body, closure, object, .. } => {
                                let body = body.clone();
                                let closure = closure.clone();
                                let object = object.clone();
                                std::thread::spawn(move || {
                                    let mut interpreter = GLOBAL_INTERPRETER.lock().unwrap();
                                    let mut local_env = Arc::new(
                                        Mutex::new(Environment::new(Some(closure)))
                                    );
                                    if let Some(obj) = object {
                                        let value = obj.lock().unwrap().clone();
                                        local_env.lock().unwrap().define("this".to_string(), value);
                                    }
                                    let _ = interpreter.visit_block(&body, &mut local_env);
                                });
                                ctx.request_repaint();
                            }

                            _ => {}
                        }
                    }
                }
                if response.hovered() && !control.cursor.is_empty() {
                    let cursor_icon = match control.cursor.to_lowercase().as_str() {
                        "hand" | "pointer" => CursorIcon::PointingHand,
                        "default" => CursorIcon::Default,
                        "crosshair" => CursorIcon::Crosshair,
                        "text" => CursorIcon::Text,
                        "move" => CursorIcon::Move,
                        "grab" => CursorIcon::Grab,
                        "grabbing" => CursorIcon::Grabbing,
                        _ => CursorIcon::Default,
                    };
                    ui.output_mut(|o| {
                        o.cursor_icon = cursor_icon;
                    });
                }
            }
            "textbox" => {
                // Use control.text as the initial value, overridden by textbox_texts if present
                let initial_text = control.text.clone();
                let text = textbox_texts.entry(control_id.clone()).or_insert(initial_text);
                ui.painter().rect_filled(rect, 0.0, control.backcolor);
                let response = ui.allocate_ui_at_rect(rect, |ui| {
                    let font_id = FontId::new(control.fontsize, get_font_family(&control.fontname));
                    if control.multiline {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let text_edit = egui::TextEdit
                                ::multiline(text)
                                .text_color(control.forecolor)
                                .font(font_id)
                                .frame(false)
                                .desired_width(control.width);
                            ui.add_sized(size, text_edit)
                        }).inner
                    } else {
                        let text_edit = egui::TextEdit
                            ::singleline(text)
                            .text_color(control.forecolor)
                            .font(font_id)
                            .frame(false)
                            .desired_width(control.width);
                        ui.add_sized(size, text_edit)
                    }
                });
                // Update CONTROLS only when the textbox loses focus and the text has changed
                if response.inner.lost_focus() && response.inner.changed() {
                    let mut controls = CONTROLS.write().unwrap();
                    if let Some(ctrl) = controls.get_mut(control_id) {
                        ctrl.text = text.clone();
                    }
                }
            }
            _ => {
                ui.painter().rect_filled(rect, 0.0, Color32::RED); // Unknown control type
            }
        }
    });
}

pub fn add_to_container(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!(
                "add_to_container() expects 2 arguments (container_id, control_id), got {}",
                args.len()
            )
        );
    }

    let container_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("add_to_container() expects a container identifier".to_string());
        }
    };
    let control_id = match &args[1] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("add_to_container() expects a control identifier".to_string());
        }
    };

    let mut controls = CONTROLS.write().unwrap();
    let mut forms = FORMS.write().unwrap();

    // Verify container exists and is a valid container type
    if let Some(container) = controls.get(&container_id) {
        if
            !matches!(
                container.control_type.as_str(),
                "groupbox" |
                    "panel" |
                    "card" |
                    "sidebar" |
                    "flowlayout" |
                    "verticallayout" |
                    "horizontallayout" |
                    "gridlayout"
            )
        {
            return Err("Identifier must be a container".to_string());
        }

        let container_pos = container.position;
        let form_id = container.form_id.clone();

        // Adjust child control’s position and form_id
        if let Some(child) = controls.get_mut(&control_id) {
            child.position = Pos2::new(
                container_pos.x + child.position.x,
                container_pos.y + child.position.y
            );
            child.form_id = form_id.clone();
        } else {
            return Err("Control not found".to_string());
        }

        // Add control to container’s children
        if let Some(container) = controls.get_mut(&container_id) {
            container.children.push(control_id.clone());
        }

        // Remove control from form’s controls_order to prevent duplicate rendering
        if let Some(form) = forms.get_mut(&form_id) {
            form.controls_order.retain(|id| id != &control_id);
        }

        Ok(Value::Null)
    } else {
        Err("Container not found".to_string())
    }
}

pub fn add_to_page(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            format!(
                "add_to_page() expects 3 arguments (pages_id, page_index, control_id), got {}",
                args.len()
            )
        );
    }

    let pages_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("add_to_page() expects a Pages identifier".to_string());
        }
    };
    let page_index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("add_to_page() expects a number for page_index".to_string());
        }
    };
    let control_id = match &args[2] {
        Value::String(id) => id.clone(),
        _ => {
            return Err("add_to_page() expects a control identifier".to_string());
        }
    };

    let mut pages_states = PAGES_STATES.write().unwrap();
    let mut controls = CONTROLS.write().unwrap();
    let mut forms = FORMS.write().unwrap();

    if let Some(state) = pages_states.get_mut(&pages_id) {
        if page_index >= state.pages.len() {
            return Err("Page index out of bounds".to_string());
        }
        if let Some(pages_control) = controls.get(&pages_id) {
            let form_id = pages_control.form_id.clone();
            if let Some(child) = controls.get_mut(&control_id) {
                child.form_id = form_id.clone();
            } else {
                return Err("Control not found".to_string());
            }
            state.pages[page_index].control_ids.push(control_id.clone());
            if let Some(form) = forms.get_mut(&form_id) {
                form.controls_order.retain(|id| id != &control_id); // Remove from form’s controls_order
            }
            Ok(Value::Null)
        } else {
            Err("Pages control not found".to_string())
        }
    } else {
        Err("Pages state not found".to_string())
    }
}

fn is_container(control_type: &str) -> bool {
    matches!(
        control_type,
        "groupbox" |
            "panel" |
            "card" |
            "sidebar" |
            "gridlayout" |
            "horizontallayout" |
            "verticallayout" |
            "flowlayout"
    )
}

fn lighten(color: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    let factor = factor.clamp(0.0, 1.0); // Ensure factor is between 0 and 1
    let r = ((r as f32) + (255.0 - (r as f32)) * factor).min(255.0) as u8;
    let g = ((g as f32) + (255.0 - (g as f32)) * factor).min(255.0) as u8;
    let b = ((b as f32) + (255.0 - (b as f32)) * factor).min(255.0) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, a)
}
