/****************************************************************************************
 * File: easyplot.rs (plotter)
 * Author: Muhammad Baba Goni
 * Created: March 22, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides simple 2D plotting capabilities (line plots, bar charts, etc.).
 *
 * Responsibilities:
 * -----------------
 * - Create simple plots based on user data.
 * - Export or render plots to the screen or file.
 * - Support basic customization (colors, labels, titles).
 *
 * Usage:
 * ------
 * Common in data analysis, debugging, and reporting tools.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use std::sync::{ Arc, Mutex };
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use minifb::{ Key, Window, WindowOptions };
use image::io::Reader as ImageReader;
use std::path::Path;

// Type alias for our result type.
pub type Result<T> = std::result::Result<T, String>;

thread_local! {
    static CURRENT_FIGURE: Arc<Mutex<Option<Arc<Mutex<Figure>>>>> = Arc::new(Mutex::new(None));
}

/// Represents a figure (canvas) on which plots are drawn.

#[derive(Debug)]
pub struct Figure {
    pub width: u32,
    pub height: u32,
    /// Plot commands to be rendered.
    pub commands: Vec<PlotCommand>,
    /// Optional title.
    pub title: Option<String>,
    /// Optional x-axis label.
    pub xlabel: Option<String>,
    /// Optional y-axis label.
    pub ylabel: Option<String>,
    /// Flag to show legend.
    pub legend: bool,
    /// Flag to show grid.
    pub grid: bool,
    /// Optional subplot configuration: (nrows, ncols)
    pub subplots: Option<(usize, usize)>,
}

/// All supported plot commands.
#[derive(Debug)]
pub enum PlotCommand {
    // Original commands.
    Line {
        x: Vec<f64>,
        y: Vec<f64>,
        color: String,
    },
    Scatter {
        x: Vec<f64>,
        y: Vec<f64>,
        color: String,
    },
    Histogram {
        data: Vec<f64>,
        bins: usize,
        color: String,
    },
    Bar {
        categories: Vec<String>,
        values: Vec<f64>,
        color: String,
    },
    Pie {
        labels: Vec<String>,
        data: Vec<f64>,
        colors: Vec<String>,
    },
    BoxPlot {
        data: Vec<Vec<f64>>,
        labels: Option<Vec<String>>,
        color: String,
    },
    ErrorBar {
        x: Vec<f64>,
        y: Vec<f64>,
        yerr: Vec<f64>,
        color: String,
    },
    Heatmap {
        data: Vec<Vec<f64>>,
        color: String,
    },
    Contour {
        data: Vec<Vec<f64>>,
        x_range: (f64, f64),
        y_range: (f64, f64),
        color: String,
    },
    Quiver {
        x: Vec<f64>,
        y: Vec<f64>,
        u: Vec<f64>,
        v: Vec<f64>,
        color: String,
    },
    Area {
        x: Vec<f64>,
        y: Vec<f64>,
        color: String,
    },
    Step {
        x: Vec<f64>,
        y: Vec<f64>,
        color: String,
    },
    Violin {
        data: Vec<Vec<f64>>,
        labels: Option<Vec<String>>,
        color: String,
    },
    // New chart types.
    StemPlot {
        x: Vec<f64>,
        y: Vec<f64>,
        marker_color: String,
        line_color: String,
    }, 
    BubbleChart {
        x: Vec<f64>,
        y: Vec<f64>,
        sizes: Vec<f64>,
        color: String,
    },
    StackedBar {
        categories: Vec<String>,
        values: Vec<Vec<f64>>,
        colors: Vec<String>,
    },
    PolarChart {
        theta: Vec<f64>,
        r: Vec<f64>,
        color: String,
    },
    Candlestick {
        timestamps: Vec<String>,
        open: Vec<f64>,
        high: Vec<f64>,
        low: Vec<f64>,
        close: Vec<f64>,
        color_up: String,
        color_down: String,
    },
    RadarChart {
        labels: Vec<String>,
        data: Vec<Vec<f64>>,
        color: String,
    },
    WaterfallChart {
        x_labels: Vec<String>,
        values: Vec<f64>,
        color: String,
    },
}

/// Updated color parser: accepts hex strings, rgb() strings, or color names.
fn color_from_string(color: &str) -> RGBColor {
    // Check for hex notation (e.g. "#ff0000")
    if color.starts_with('#') {
        let hex = &color[1..];
        if hex.len() == 6 {
            if
                let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                )
            {
                return RGBColor(r, g, b);
            }
        }
    }
    // Check for rgb(...) format.
    if color.to_lowercase().starts_with("rgb(") && color.ends_with(')') {
        let inner = &color[4..color.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            if
                let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].trim().parse::<u8>(),
                    parts[1].trim().parse::<u8>(),
                    parts[2].trim().parse::<u8>(),
                )
            {
                return RGBColor(r, g, b);
            }
        }
    }
    // Fallback to named colors.
    match color.to_lowercase().as_str() {
        "red" => RED,
        "blue" => BLUE,
        "green" => GREEN,
        "black" => BLACK,
        "yellow" => YELLOW,
        "purple" => RGBColor(128, 0, 128),
        "orange" => RGBColor(255, 165, 0),
        _ => BLUE, // default fallback
    }
}

/// mpl_figure([width, height])
/// Creates a new figure with optional dimensions (defaults to 800×600) and initializes extra fields.
pub fn mpl_figure(args: Vec<Value>) -> Result<Value> {
    let (width, height) = if args.len() == 0 {
        (800, 600)
    } else if args.len() == 2 {
        let w = match &args[0] {
            Value::Number(n) => *n as u32,
            _ => {
                return Err("mpl_figure() expects a number for width".to_string());
            }
        };
        let h = match &args[1] {
            Value::Number(n) => *n as u32,
            _ => {
                return Err("mpl_figure() expects a number for height".to_string());
            }
        };
        (w, h)
    } else {
        return Err(format!("mpl_figure() expects 0 or 2 arguments, got {}", args.len()));
    };

    let fig = Figure {
        width,
        height,
        commands: Vec::new(),
        title: None,
        xlabel: None,
        ylabel: None,
        legend: false,
        grid: false,
        subplots: None,
    };
    let fig_arc = Arc::new(Mutex::new(fig));
    CURRENT_FIGURE.with(|cf| {
        let mut guard = cf.lock().unwrap();
        *guard = Some(fig_arc.clone());
    });
    Ok(Value::Figure(fig_arc))
}

/// mpl_title(title)
/// Sets the title of the current figure.
pub fn mpl_title(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mpl_title() expects 1 argument, got {}", args.len()));
    }
    let title = extract_string(&args[0], "mpl_title()")?;
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().title = Some(title);
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_xlabel(label)
/// Sets the x-axis label of the current figure.
pub fn mpl_xlabel(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mpl_xlabel() expects 1 argument, got {}", args.len()));
    }
    let label = extract_string(&args[0], "mpl_xlabel()")?;
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().xlabel = Some(label);
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_ylabel(label)
/// Sets the y-axis label of the current figure.
pub fn mpl_ylabel(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mpl_ylabel() expects 1 argument, got {}", args.len()));
    }
    let label = extract_string(&args[0], "mpl_ylabel()")?;
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().ylabel = Some(label);
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_legend(show)
/// Toggles legend display (expects a boolean; default true if omitted).
pub fn mpl_legend(args: Vec<Value>) -> Result<Value> {
    let show = if args.len() == 0 {
        true
    } else if args.len() == 1 {
        match &args[0] {
            Value::Bool(b) => *b,
            _ => {
                return Err("mpl_legend() expects a boolean".to_string());
            }
        }
    } else {
        return Err(format!("mpl_legend() expects 0 or 1 arguments, got {}", args.len()));
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().legend = show;
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_grid(on)
/// Toggles grid display (expects a boolean; default true if omitted).
pub fn mpl_grid(args: Vec<Value>) -> Result<Value> {
    let on = if args.len() == 0 {
        true
    } else if args.len() == 1 {
        match &args[0] {
            Value::Bool(b) => *b,
            _ => {
                return Err("mpl_grid() expects a boolean".to_string());
            }
        }
    } else {
        return Err(format!("mpl_grid() expects 0 or 1 arguments, got {}", args.len()));
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().grid = on;
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_clear()
/// Clears the current figure.
pub fn mpl_clear(args: Vec<Value>) -> Result<Value> {
    if args.len() != 0 {
        return Err(format!("mpl_clear() expects 0 arguments, got {}", args.len()));
    }
    CURRENT_FIGURE.with(|cf| {
        let mut guard = cf.lock().unwrap();
        *guard = None;
    });
    Ok(Value::Null)
}

/// mpl_subplots(nrows, ncols)
/// Configures the current figure for subplots arranged in a grid.
pub fn mpl_subplots(args: Vec<Value>) -> Result<Value> {
    if args.len() != 2 {
        return Err(format!("mpl_subplots() expects 2 arguments, got {}", args.len()));
    }
    let nrows = match &args[0] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("mpl_subplots() expects a number for rows".to_string());
        }
    };
    let ncols = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("mpl_subplots() expects a number for columns".to_string());
        }
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().subplots = Some((nrows, ncols));
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// --- Plotting Functions for Various Chart Types ---

/// mpl_plot(x_values, y_values, [color])
pub fn mpl_plot(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_plot() expects 2 or 3 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_plot() x values")?;
    let y = extract_number_array(&args[1], "mpl_plot() y values")?;
    if x.len() != y.len() {
        return Err("mpl_plot() expects x and y arrays of equal length".to_string());
    }
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_plot() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Line { x, y, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_scatter(x_values, y_values, [color])
pub fn mpl_scatter(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_scatter() expects 2 or 3 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_scatter() x values")?;
    let y = extract_number_array(&args[1], "mpl_scatter() y values")?;
    if x.len() != y.len() {
        return Err("mpl_scatter() expects x and y arrays of equal length".to_string());
    }
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_scatter() color")?
    } else {
        "red".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Scatter { x, y, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_hist(data, [bins], [color])
pub fn mpl_hist(args: Vec<Value>) -> Result<Value> {
    if args.len() < 1 || args.len() > 3 {
        return Err(format!("mpl_hist() expects 1 to 3 arguments, got {}", args.len()));
    }
    let data = extract_number_array(&args[0], "mpl_hist() data")?;
    let bins = if args.len() >= 2 {
        match &args[1] {
            Value::Number(n) => *n as usize,
            _ => {
                return Err("mpl_hist() bins must be a number".to_string());
            }
        }
    } else {
        10
    };
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_hist() color")?
    } else {
        "green".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Histogram { data, bins, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_bar(categories, values, [color])
pub fn mpl_bar(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_bar() expects 2 or 3 arguments, got {}", args.len()));
    }
    let categories = extract_string_array(&args[0], "mpl_bar() categories")?;
    let values = extract_number_array(&args[1], "mpl_bar() values")?;
    if categories.len() != values.len() {
        return Err("mpl_bar() expects categories and values arrays of equal length".to_string());
    }
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_bar() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Bar { categories, values, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_pie(labels, data, [colors])
pub fn mpl_pie(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_pie() expects 2 or 3 arguments, got {}", args.len()));
    }
    let labels = extract_string_array(&args[0], "mpl_pie() labels")?;
    let data = extract_number_array(&args[1], "mpl_pie() data")?;
    if labels.len() != data.len() {
        return Err("mpl_pie() expects labels and data arrays of equal length".to_string());
    }
    let colors = if args.len() == 3 {
        extract_string_array(&args[2], "mpl_pie() colors")?
    } else {
        (0..data.len())
            .map(|i| {
                let default = ["red", "blue", "green", "yellow", "purple", "orange"];
                default[i % default.len()].to_string()
            })
            .collect()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Pie { labels, data, colors });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_boxplot(data, [labels], [color])
pub fn mpl_boxplot(args: Vec<Value>) -> Result<Value> {
    if args.len() < 1 || args.len() > 3 {
        return Err(format!("mpl_boxplot() expects 1 to 3 arguments, got {}", args.len()));
    }
    let data = extract_number_matrix(&args[0], "mpl_boxplot() data")?;
    let labels = if args.len() >= 2 {
        Some(extract_string_array(&args[1], "mpl_boxplot() labels")?)
    } else {
        None
    };
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_boxplot() color")?
    } else {
        "purple".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::BoxPlot { data, labels, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_errorbar(x, y, yerr, [color])
pub fn mpl_errorbar(args: Vec<Value>) -> Result<Value> {
    if args.len() < 3 || args.len() > 4 {
        return Err(format!("mpl_errorbar() expects 3 or 4 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_errorbar() x")?;
    let y = extract_number_array(&args[1], "mpl_errorbar() y")?;
    let yerr = extract_number_array(&args[2], "mpl_errorbar() yerr")?;
    if x.len() != y.len() || y.len() != yerr.len() {
        return Err("mpl_errorbar() expects x, y, and yerr arrays of equal length".to_string());
    }
    let color = if args.len() == 4 {
        extract_string(&args[3], "mpl_errorbar() color")?
    } else {
        "black".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::ErrorBar { x, y, yerr, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_heatmap(data, [color])
pub fn mpl_heatmap(args: Vec<Value>) -> Result<Value> {
    if args.len() < 1 || args.len() > 2 {
        return Err(format!("mpl_heatmap() expects 1 or 2 arguments, got {}", args.len()));
    }
    let data = extract_number_matrix(&args[0], "mpl_heatmap() data")?;
    let color = if args.len() == 2 {
        extract_string(&args[1], "mpl_heatmap() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Heatmap { data, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_contour(data, x_range, y_range, color)
pub fn mpl_contour(args: Vec<Value>) -> Result<Value> {
    if args.len() != 4 {
        return Err(format!("mpl_contour() expects 4 arguments, got {}", args.len()));
    }
    let data = extract_number_matrix(&args[0], "mpl_contour() data")?;
    let x_range = extract_range(&args[1], "mpl_contour() x_range")?;
    let y_range = extract_range(&args[2], "mpl_contour() y_range")?;
    let color = extract_string(&args[3], "mpl_contour() color")?;
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc
                .lock()
                .unwrap()
                .commands.push(PlotCommand::Contour { data, x_range, y_range, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_quiver(x, y, u, v, [color])
pub fn mpl_quiver(args: Vec<Value>) -> Result<Value> {
    if args.len() < 4 || args.len() > 5 {
        return Err(format!("mpl_quiver() expects 4 or 5 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_quiver() x")?;
    let y = extract_number_array(&args[1], "mpl_quiver() y")?;
    let u = extract_number_array(&args[2], "mpl_quiver() u")?;
    let v = extract_number_array(&args[3], "mpl_quiver() v")?;
    if x.len() != y.len() || x.len() != u.len() || x.len() != v.len() {
        return Err("mpl_quiver() expects x, y, u, and v arrays of equal length".to_string());
    }
    let color = if args.len() == 5 {
        extract_string(&args[4], "mpl_quiver() color")?
    } else {
        "black".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Quiver { x, y, u, v, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_area(x, y, [color])
pub fn mpl_area(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_area() expects 2 or 3 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_area() x")?;
    let y = extract_number_array(&args[1], "mpl_area() y")?;
    if x.len() != y.len() {
        return Err("mpl_area() expects x and y arrays of equal length".to_string());
    }
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_area() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Area { x, y, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_step(x, y, [color])
pub fn mpl_step(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_step() expects 2 or 3 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_step() x")?;
    let y = extract_number_array(&args[1], "mpl_step() y")?;
    if x.len() != y.len() {
        return Err("mpl_step() expects x and y arrays of equal length".to_string());
    }
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_step() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Step { x, y, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_violin(data, [labels], [color])
pub fn mpl_violin(args: Vec<Value>) -> Result<Value> {
    if args.len() < 1 || args.len() > 3 {
        return Err(format!("mpl_violin() expects 1 to 3 arguments, got {}", args.len()));
    }
    let data = extract_number_matrix(&args[0], "mpl_violin() data")?;
    let labels = if args.len() >= 2 {
        Some(extract_string_array(&args[1], "mpl_violin() labels")?)
    } else {
        None
    };
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_violin() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Violin { data, labels, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// --- New Chart Types ---

/// mpl_stem(x_values, y_values, [marker_color], [line_color])
pub fn mpl_stem(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(format!("mpl_stem() expects 2 to 4 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_stem() x values")?;
    let y = extract_number_array(&args[1], "mpl_stem() y values")?;
    if x.len() != y.len() {
        return Err("mpl_stem() expects x and y arrays of equal length".to_string());
    }
    let marker_color = if args.len() >= 3 {
        extract_string(&args[2], "mpl_stem() marker_color")?
    } else {
        "black".to_string()
    };
    let line_color = if args.len() == 4 {
        extract_string(&args[3], "mpl_stem() line_color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc
                .lock()
                .unwrap()
                .commands.push(PlotCommand::StemPlot { x, y, marker_color, line_color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_bubble(x_values, y_values, sizes, [color])
pub fn mpl_bubble(args: Vec<Value>) -> Result<Value> {
    if args.len() < 3 || args.len() > 4 {
        return Err(format!("mpl_bubble() expects 3 or 4 arguments, got {}", args.len()));
    }
    let x = extract_number_array(&args[0], "mpl_bubble() x values")?;
    let y = extract_number_array(&args[1], "mpl_bubble() y values")?;
    let sizes = extract_number_array(&args[2], "mpl_bubble() sizes")?;
    if x.len() != y.len() || x.len() != sizes.len() {
        return Err("mpl_bubble() expects x, y, and sizes arrays of equal length".to_string());
    }
    let color = if args.len() == 4 {
        extract_string(&args[3], "mpl_bubble() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::BubbleChart { x, y, sizes, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_stackedbar(categories, values, [colors])
/// Expects categories (array of strings) and values (2D array: each inner array is a series for that category).
pub fn mpl_stackedbar(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_stackedbar() expects 2 or 3 arguments, got {}", args.len()));
    }
    let categories = extract_string_array(&args[0], "mpl_stackedbar() categories")?;
    let values = extract_number_matrix(&args[1], "mpl_stackedbar() values")?;
    if values.len() != categories.len() {
        return Err(
            "mpl_stackedbar() expects the number of rows in values to match the number of categories".to_string()
        );
    }
    let colors = if args.len() == 3 {
        extract_string_array(&args[2], "mpl_stackedbar() colors")?
    } else {
        (0..categories.len())
            .map(|i| {
                let default = ["red", "blue", "green", "yellow", "purple", "orange"];
                default[i % default.len()].to_string()
            })
            .collect()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc
                .lock()
                .unwrap()
                .commands.push(PlotCommand::StackedBar { categories, values, colors });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_polar(theta, r, [color])
pub fn mpl_polar(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_polar() expects 2 or 3 arguments, got {}", args.len()));
    }
    let theta = extract_number_array(&args[0], "mpl_polar() theta")?;
    let r = extract_number_array(&args[1], "mpl_polar() r")?;
    if theta.len() != r.len() {
        return Err("mpl_polar() expects theta and r arrays of equal length".to_string());
    }
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_polar() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::PolarChart { theta, r, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_candlestick(timestamps, open, high, low, close, [color_up], [color_down])
pub fn mpl_candlestick(args: Vec<Value>) -> Result<Value> {
    if args.len() < 5 || args.len() > 7 {
        return Err(format!("mpl_candlestick() expects 5 to 7 arguments, got {}", args.len()));
    }
    let timestamps = extract_string_array(&args[0], "mpl_candlestick() timestamps")?;
    let open = extract_number_array(&args[1], "mpl_candlestick() open")?;
    let high = extract_number_array(&args[2], "mpl_candlestick() high")?;
    let low = extract_number_array(&args[3], "mpl_candlestick() low")?;
    let close = extract_number_array(&args[4], "mpl_candlestick() close")?;
    if
        timestamps.len() != open.len() ||
        open.len() != high.len() ||
        high.len() != low.len() ||
        low.len() != close.len()
    {
        return Err("mpl_candlestick() expects all arrays to be of equal length".to_string());
    }
    let color_up = if args.len() >= 6 {
        extract_string(&args[5], "mpl_candlestick() color_up")?
    } else {
        "green".to_string()
    };
    let color_down = if args.len() == 7 {
        extract_string(&args[6], "mpl_candlestick() color_down")?
    } else {
        "red".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::Candlestick {
                timestamps,
                open,
                high,
                low,
                close,
                color_up,
                color_down,
            });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_radar(labels, data, [color])
pub fn mpl_radar(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_radar() expects 2 or 3 arguments, got {}", args.len()));
    }
    let labels = extract_string_array(&args[0], "mpl_radar() labels")?;
    let data = extract_number_matrix(&args[1], "mpl_radar() data")?;
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_radar() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc.lock().unwrap().commands.push(PlotCommand::RadarChart { labels, data, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// mpl_waterfall(x_labels, values, [color])
pub fn mpl_waterfall(args: Vec<Value>) -> Result<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(format!("mpl_waterfall() expects 2 or 3 arguments, got {}", args.len()));
    }
    let x_labels = extract_string_array(&args[0], "mpl_waterfall() x_labels")?;
    let values = extract_number_array(&args[1], "mpl_waterfall() values")?;
    if x_labels.len() != values.len() {
        return Err(
            "mpl_waterfall() expects x_labels and values arrays of equal length".to_string()
        );
    }
    let color = if args.len() == 3 {
        extract_string(&args[2], "mpl_waterfall() color")?
    } else {
        "blue".to_string()
    };
    CURRENT_FIGURE.with(|cf| {
        if let Some(fig_arc) = &*cf.lock().unwrap() {
            fig_arc
                .lock()
                .unwrap()
                .commands.push(PlotCommand::WaterfallChart { x_labels, values, color });
            Ok(Value::Null)
        } else {
            Err("No active figure. Call mpl_figure() first.".to_string())
        }
    })
}

/// --- Rendering the Figure ---

/// mpl_savefig(filename)
/// Renders the current figure using Plotters and saves it to an image file.
/// If subplots are configured, the drawing area is partitioned accordingly.
/// Helper function to compute Cartesian ranges for a plot command.
fn get_cartesian_ranges(cmd: &PlotCommand) -> Option<((f64, f64), (f64, f64))> {
    match cmd {
        PlotCommand::Line { x, y, .. } |
        PlotCommand::Scatter { x, y, .. } |
        PlotCommand::ErrorBar { x, y, .. } |
        PlotCommand::Area { x, y, .. } |
        PlotCommand::Step { x, y, .. } |
        PlotCommand::StemPlot { x, y, .. } |
        PlotCommand::BubbleChart { x, y, .. } |
        PlotCommand::Quiver { x, y, .. } => {
            if x.is_empty() || y.is_empty() { None } else {
                let x_min = x.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let x_max = x.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let y_min = y.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let y_max = y.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                Some(((x_min, x_max), (y_min, y_max)))
            }
        }
        PlotCommand::Histogram { data, bins, .. } => {
            if data.is_empty() { None } else {
                let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let bin_width = (max - min) / (*bins as f64);
                let mut counts = vec![0; *bins];
                for &val in data {
                    let bin = ((val - min) / bin_width).floor() as usize;
                    if bin >= *bins { counts[*bins - 1] += 1; } else { counts[bin] += 1; }
                }
                let y_max = *counts.iter().max().unwrap_or(&0) as f64;
                Some(((min, max), (0.0, y_max)))
            }
        }
        PlotCommand::Bar { values, .. } => {
            if values.is_empty() { None } else {
                let x_max = values.len() as f64;
                let y_min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let y_max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                Some(((0.0, x_max), (y_min.max(0.0), y_max)))
            }
        }
        PlotCommand::BoxPlot { data, .. } => {
            if data.is_empty() { None } else {
                let x_max = data.len() as f64;
                let mut y_min = f64::INFINITY;
                let mut y_max = f64::NEG_INFINITY;
                for dataset in data {
                    if !dataset.is_empty() {
                        y_min = y_min.min(dataset.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
                        y_max = y_max.max(dataset.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
                    }
                }
                if y_min == f64::INFINITY { None } else { Some(((0.0, x_max), (y_min, y_max))) }
            }
        }
        PlotCommand::Heatmap { data, .. } => {
            if data.is_empty() || data[0].is_empty() { None } else {
                let x_max = data.len() as f64;
                let y_max = data[0].len() as f64;
                Some(((0.0, x_max), (0.0, y_max)))
            }
        }
        PlotCommand::Contour { x_range, y_range, .. } => Some((*x_range, *y_range)),
        PlotCommand::Violin { data, .. } => {
            if data.is_empty() { None } else {
                let x_max = data.len() as f64;
                let mut y_min = f64::INFINITY;
                let mut y_max = f64::NEG_INFINITY;
                for dataset in data {
                    if !dataset.is_empty() {
                        y_min = y_min.min(dataset.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
                        y_max = y_max.max(dataset.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
                    }
                }
                if y_min == f64::INFINITY { None } else { Some(((0.0, x_max), (y_min, y_max))) }
            }
        }
        PlotCommand::StackedBar { values, .. } => {
            if values.is_empty() { None } else {
                let x_max = values[0].len() as f64;
                let mut y_max: f64 = 0.0;
                for i in 0..values[0].len() {
                    let sum = values.iter().map(|v| v[i]).sum::<f64>();
                    y_max = y_max.max(sum);
                }
                Some(((0.0, x_max), (0.0, y_max)))
            }
        }
        PlotCommand::Candlestick { open, high, low, close, .. } => {
            if open.is_empty() { None } else {
                let x_max = open.len() as f64;
                let y_min = low.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let y_max = high.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                Some(((0.0, x_max), (y_min, y_max)))
            }
        }
        PlotCommand::WaterfallChart { values, .. } => {
            if values.is_empty() { None } else {
                let x_max = values.len() as f64;
                let mut cumulative = 0.0;
                let mut y_min: f64 = 0.0;
                let mut y_max: f64 = 0.0;
                for &val in values {
                    cumulative += val;
                    y_min = y_min.min(cumulative as f64);
                    y_max = y_max.max(cumulative as f64);

                }
                Some(((0.0, x_max), (y_min, y_max)))
            }
        }
        PlotCommand::PolarChart { theta, r, .. } => {
            let (x, y): (Vec<f64>, Vec<f64>) = theta.iter().zip(r.iter()).map(|(&t, &r)| {
                let rad = t.to_radians();
                (r * rad.cos(), r * rad.sin())
            }).unzip();
            if x.is_empty() { None } else {
                let x_min = x.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let x_max = x.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let y_min = y.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let y_max = y.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                Some(((x_min, x_max), (y_min, y_max)))
            }
        }
        _ => None,
    }
}

/// mpl_savefig(filename)
/// Renders the current figure to the specified file, supporting all 20 chart types.
pub fn mpl_savefig(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mpl_savefig() expects 1 argument, got {}", args.len()));
    }
    let filename = extract_string(&args[0], "mpl_savefig() filename")?;

    let fig_arc = CURRENT_FIGURE.with(|cf| cf.lock().unwrap().clone())
        .ok_or("No active figure. Call mpl_figure() first.")?;
    let fig = fig_arc.lock().unwrap();
    let root = BitMapBackend::new(&filename, (fig.width, fig.height)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| format!("mpl_savefig() error: {}", e))?;

    if let Some((nrows, ncols)) = fig.subplots {
        let areas = root.split_evenly((nrows, ncols));
        for (i, cell_area) in areas.iter().enumerate() {
            if i >= fig.commands.len() { continue; }
            let cmd = &fig.commands[i];
            if let Some(((x_min, x_max), (y_min, y_max))) = get_cartesian_ranges(cmd) {
                let mut chart = ChartBuilder::on(cell_area)
                    .margin(10)
                    .set_label_area_size(LabelAreaPosition::Left, 20)
                    .set_label_area_size(LabelAreaPosition::Bottom, 20)
                    .build_cartesian_2d(x_min..x_max, y_min..y_max)
                    .map_err(|e| format!("mpl_savefig() error: {}", e))?;
                if fig.grid {
                    chart.configure_mesh().draw().map_err(|e| format!("mpl_savefig() error: {}", e))?;
                }
                render_cartesian_command(&mut chart, cmd)?;
            } else {
                render_non_cartesian_command(cell_area, cmd, fig.width, fig.height)?;
            }
        }
    } else {
        let mut cartesian_commands = Vec::new();
        let mut non_cartesian_commands = Vec::new();
        for cmd in &fig.commands {
            if get_cartesian_ranges(cmd).is_some() {
                cartesian_commands.push(cmd);
            } else {
                non_cartesian_commands.push(cmd);
            }
        }

        if !cartesian_commands.is_empty() {
            let mut x_min = f64::INFINITY;
            let mut x_max = f64::NEG_INFINITY;
            let mut y_min = f64::INFINITY;
            let mut y_max = f64::NEG_INFINITY;
            for cmd in &cartesian_commands {
                if let Some(((x0, x1), (y0, y1))) = get_cartesian_ranges(cmd) {
                    x_min = x_min.min(x0);
                    x_max = x_max.max(x1);
                    y_min = y_min.min(y0);
                    y_max = y_max.max(y1);
                }
            }
            let mut chart = ChartBuilder::on(&root)
                .margin(20)
                .caption(fig.title.clone().unwrap_or_else(|| "Plot".to_string()), ("sans-serif", 20))
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(x_min..x_max, y_min..y_max)
                .map_err(|e| format!("mpl_savefig() error: {}", e))?;
            if fig.grid {
                chart.configure_mesh().draw().map_err(|e| format!("mpl_savefig() error: {}", e))?;
            }
            for cmd in cartesian_commands {
                render_cartesian_command(&mut chart, cmd)?;
            }
        }

        for cmd in non_cartesian_commands {
            render_non_cartesian_command(&root, cmd, fig.width, fig.height)?;
        }
    }

    root.present().map_err(|e| format!("mpl_savefig() error: {}", e))?;
    Ok(Value::Null)
}

/// Helper function to render Cartesian-based plot commands.
fn render_cartesian_command(chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordf64, RangedCoordf64>>, cmd: &PlotCommand) -> Result<()> {
    match cmd {
        PlotCommand::Line { x, y, color } => {
            let pts: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)).collect();
            chart.draw_series(LineSeries::new(pts, &color_from_string(color)));
        }
        PlotCommand::Scatter { x, y, color } => {
            let pts: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)).collect();
            chart.draw_series(pts.iter().map(|&(a, b)| Circle::new((a, b), 3, ShapeStyle::from(&color_from_string(color)).filled())));
        }
        PlotCommand::Histogram { data, bins, color } => {
            let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let bin_width = (max - min) / (*bins as f64);
            let mut counts = vec![0; *bins];
            for &val in data {
                let bin = ((val - min) / bin_width).floor() as usize;
                if bin >= *bins { counts[*bins - 1] += 1; } else { counts[bin] += 1; }
            }
            for i in 0..*bins {
                let x0 = min + i as f64 * bin_width;
                let x1 = x0 + bin_width;
                let y = counts[i] as f64;
                chart.draw_series(std::iter::once(Rectangle::new([(x0, 0.0), (x1, y)], ShapeStyle::from(&color_from_string(color)).filled())));
            }
        }
        PlotCommand::Bar { values, color, .. } => {
            let bar_width = 0.8;
            for (i, &val) in values.iter().enumerate() {
                let x0 = i as f64 - bar_width / 2.0;
                let x1 = i as f64 + bar_width / 2.0;
                let y0 = 0.0;
                let y1 = val;
                chart.draw_series(std::iter::once(Rectangle::new([(x0, y0), (x1, y1)], ShapeStyle::from(&color_from_string(color)).filled())));
            }
        }
        PlotCommand::BoxPlot { data, color, .. } => {
            for (i, dataset) in data.iter().enumerate() {
                if dataset.is_empty() { continue; }
                let mut sorted = dataset.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let q1 = sorted[(sorted.len() as f64 * 0.25) as usize];
                let median = sorted[sorted.len() / 2];
                let q3 = sorted[(sorted.len() as f64 * 0.75) as usize];
                let iqr = q3 - q1;
                let whisker_low = sorted.iter().find(|&&x| x >= q1 - 1.5 * iqr).unwrap_or(&sorted[0]);
                let whisker_high = sorted.iter().rev().find(|&&x| x <= q3 + 1.5 * iqr).unwrap_or(&sorted[sorted.len() - 1]);
                let x = i as f64;
                chart.draw_series(LineSeries::new(vec![(x, *whisker_low), (x, *whisker_high)], &color_from_string(color)));
                chart.draw_series(std::iter::once(Rectangle::new([(x - 0.2, q1), (x + 0.2, q3)], ShapeStyle::from(&color_from_string(color)).filled())));
                chart.draw_series(LineSeries::new(vec![(x - 0.2, median), (x + 0.2, median)], &color_from_string(color)));
            }
        }
        PlotCommand::ErrorBar { x, y, yerr, color } => {
            for (&xi, (&yi, &yerri)) in x.iter().zip(y.iter().zip(yerr.iter())) {
                chart.draw_series(LineSeries::new(vec![(xi, yi - yerri), (xi, yi + yerri)], &color_from_string(color)));
                chart.draw_series(std::iter::once(Circle::new((xi, yi), 3, ShapeStyle::from(&color_from_string(color)).filled())));
            }
        }
        PlotCommand::Heatmap { data, color } => {
            for (i, row) in data.iter().enumerate() {
                for (j, &val) in row.iter().enumerate() {
                    let intensity = (val / data.iter().flat_map(|r| r.iter()).fold(f64::NEG_INFINITY, |a, &b| a.max(b))).min(1.0);
                    let c = color_from_string(color);
                    let blended = RGBColor(
                        (c.0 as f64 * intensity + 255.0 * (1.0 - intensity)) as u8,
                        (c.1 as f64 * intensity + 255.0 * (1.0 - intensity)) as u8,
                        (c.2 as f64 * intensity + 255.0 * (1.0 - intensity)) as u8,
                    );
                    chart.draw_series(std::iter::once(Rectangle::new([(i as f64, j as f64), (i as f64 + 1.0, j as f64 + 1.0)], blended.filled())));
                }
            }
        }
        PlotCommand::Contour { data, x_range, y_range, color } => {
            // Simplified: draw a single contour line at the median value
            let flat: Vec<f64> = data.iter().flat_map(|r| r.iter().copied()).collect();
            let level = flat[flat.len() / 2];
            for i in 0..data.len() - 1 {
                for j in 0..data[0].len() - 1 {
                    if (data[i][j] - level) * (data[i + 1][j + 1] - level) < 0.0 {
                        let x = x_range.0 + (i as f64 / (data.len() - 1) as f64) * (x_range.1 - x_range.0);
                        let y = y_range.0 + (j as f64 / (data[0].len() - 1) as f64) * (y_range.1 - y_range.0);
                        chart.draw_series(std::iter::once(Circle::new((x, y), 1, ShapeStyle::from(&color_from_string(color)).filled())));
                    }
                }
            }
        }
        PlotCommand::Quiver { x, y, u, v, color } => {
            for (((&xi, &yi), &ui), &vi) in x.iter().zip(y.iter()).zip(u.iter()).zip(v.iter()) {
                chart.draw_series(LineSeries::new(vec![(xi, yi), (xi + ui, yi + vi)], &color_from_string(color)));
                chart.draw_series(std::iter::once(Circle::new((xi + ui, yi + vi), 2, ShapeStyle::from(&color_from_string(color)).filled())));
            }
        }
        PlotCommand::Area { x, y, color } => {
            chart.draw_series(AreaSeries::new(x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)), 0.0, &color_from_string(color).mix(0.5)));
        }
        PlotCommand::Step { x, y, color } => {
            let mut pts = Vec::new();
            for i in 0..x.len() - 1 {
                pts.push((x[i], y[i]));
                pts.push((x[i + 1], y[i]));
            }
            if !x.is_empty() { pts.push((x[x.len() - 1], y[y.len() - 1])); }
            chart.draw_series(LineSeries::new(pts, &color_from_string(color)));
        }
        PlotCommand::Violin { data, color, .. } => {
            for (i, dataset) in data.iter().enumerate() {
                if dataset.is_empty() { continue; }
                let mut sorted = dataset.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let density = 0.1; // Simplified density width
                let x = i as f64;
                for &y in dataset {
                    chart.draw_series(LineSeries::new(vec![(x - density, y), (x + density, y)], &color_from_string(color)));
                }
            }
        }
        PlotCommand::StemPlot { x, y, marker_color, line_color } => {
            for (&xi, &yi) in x.iter().zip(y.iter()) {
                chart.draw_series(LineSeries::new(vec![(xi, 0.0), (xi, yi)], &color_from_string(line_color)));
                chart.draw_series(std::iter::once(Circle::new((xi, yi), 3, ShapeStyle::from(&color_from_string(marker_color)).filled())));
            }
        }
        PlotCommand::BubbleChart { x, y, sizes, color } => {
            let max_size = sizes.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            for ((&xi, &yi), &si) in x.iter().zip(y.iter()).zip(sizes.iter()) {
                let radius = (si / max_size * 10.0).max(2.0) as i32;
                chart.draw_series(std::iter::once(Circle::new((xi, yi), radius, ShapeStyle::from(&color_from_string(color)).filled())));
            }
        }
        PlotCommand::StackedBar { values, colors, .. } => {
            let bar_width = 0.8;
            for j in 0..values[0].len() {
                let mut y_bottom = 0.0;
                for (i, series) in values.iter().enumerate() {
                    let val = series[j];
                    let x0 = j as f64 - bar_width / 2.0;
                    let x1 = j as f64 + bar_width / 2.0;
                    chart.draw_series(std::iter::once(Rectangle::new([(x0, y_bottom), (x1, y_bottom + val)], ShapeStyle::from(&color_from_string(&colors[i % colors.len()])).filled())));
                    y_bottom += val;
                }
            }
        }
        PlotCommand::Candlestick { open, high, low, close, color_up, color_down, .. } => {
            for (i, (((&o, &h), &l), &c)) in open.iter().zip(high.iter()).zip(low.iter()).zip(close.iter()).enumerate() {
                let x = i as f64;
                let color = if c >= o { &color_from_string(color_up) } else { &color_from_string(color_down) };
                chart.draw_series(LineSeries::new(vec![(x, l), (x, h)], color));
                chart.draw_series(std::iter::once(Rectangle::new([(x - 0.2, o), (x + 0.2, c)], ShapeStyle::from(color).filled())));
            }
        }
        PlotCommand::WaterfallChart { values, color, .. } => {
            let bar_width = 0.8;
            let mut cumulative = 0.0;
            for (i, &val) in values.iter().enumerate() {
                let x0 = i as f64 - bar_width / 2.0;
                let x1 = i as f64 + bar_width / 2.0;
                let y0 = cumulative;
                cumulative += val;
                let y1 = cumulative;
                chart.draw_series(std::iter::once(Rectangle::new([(x0, y0), (x1, y1)], ShapeStyle::from(&color_from_string(color)).filled())));
            }
        }
        PlotCommand::PolarChart { theta, r, color } => {
            let pts: Vec<(f64, f64)> = theta.iter().zip(r.iter()).map(|(&t, &r)| {
                let rad = t.to_radians();
                (r * rad.cos(), r * rad.sin())
            }).collect();
            chart.draw_series(LineSeries::new(pts, &color_from_string(color)));
        }
        _ => {}
    }
    Ok(())
}

/// Helper function to render non-Cartesian plot commands.
fn render_non_cartesian_command(area: &DrawingArea<BitMapBackend, plotters::coord::Shift>, cmd: &PlotCommand, width: u32, height: u32) -> Result<()> {
    match cmd {
        PlotCommand::Pie { data, colors, .. } => {
            let total = data.iter().sum::<f64>();
            let center = (width / 2, height / 2);
            let radius = (width.min(height) / 2) as f64 * 0.8;
            let mut start_angle = 0.0;
            for (i, &val) in data.iter().enumerate() {
                let angle = (val / total) * 2.0 * std::f64::consts::PI;
                let mut points = vec![(center.0 as f64, center.1 as f64)];
                let steps = 20;
                for j in 0..=steps {
                    let a = start_angle + angle * (j as f64 / steps as f64);
                    points.push((center.0 as f64 + radius * a.cos(), center.1 as f64 + radius * a.sin()));
                }
                // Convert each (f64, f64) point to (i32, i32), perhaps rounding the float values.
                let int_points: Vec<(i32, i32)> = points
                .iter()
                .map(|&(x, y)| (x.round() as i32, y.round() as i32))
                .collect();

                area.draw(&Polygon::new(
                int_points,
                ShapeStyle::from(&color_from_string(&colors[i % colors.len()])).filled()
                ));

                start_angle += angle;
            }
        }
        PlotCommand::RadarChart { data, color, labels } => {
            let center = (width / 2, height / 2);
            let radius = (width.min(height) / 2) as f64 * 0.8;
            for series in data {
                let max_val = series.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let n = labels.len();
                let mut points = Vec::new();
                for (i, &val) in series.iter().enumerate() {
                    let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                    let r = (val / max_val) * radius;
                    points.push((center.0 as f64 + r * angle.cos(), center.1 as f64 + r * angle.sin()));
                }
                points.push(points[0]); // Close the polygon
                
                let int_points: Vec<(i32, i32)> = points
                .iter()
                .map(|&(x, y)| (x.round() as i32, y.round() as i32))
                .collect();

                let _ = area.draw(&Polygon::new(
                int_points,
                ShapeStyle::from(&color_from_string(color)).filled().color.mix(0.5)
                ));

            }
        }
        _ => {}
    }
    Ok(())
}

/// mpl_show()
/// Displays the current figure by saving it to a temporary file and showing it in a window.
pub fn mpl_show(args: Vec<Value>) -> Result<Value> {
    if !args.is_empty() {
        return Err(format!("mpl_show() expects 0 arguments, got {}", args.len()));
    }

    let temp_filename = "temp_plot.png";
    mpl_savefig(vec![Value::String(temp_filename.to_string())])?;

    let img = ImageReader::open(Path::new(temp_filename))
        .map_err(|e| format!("Failed to open image '{}': {}", temp_filename, e))?
        .decode()
        .map_err(|e| format!("Failed to decode image '{}': {}", temp_filename, e))?
        .into_rgba8();
    let (width, height) = img.dimensions();

    let buffer: Vec<u32> = img.pixels().map(|p| {
        let r = p[0] as u32;
        let g = p[1] as u32;
        let b = p[2] as u32;
        let a = p[3] as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }).collect();

    let mut window = Window::new(
        "Plot Display",
        width as usize,
        height as usize,
        WindowOptions::default()
    ).map_err(|e| format!("Failed to create window: {}", e))?;

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, width as usize, height as usize)
            .map_err(|e| format!("Failed to update window: {}", e))?;
    }

    CURRENT_FIGURE.with(|cf| {
        let mut guard = cf.lock().unwrap();
        *guard = None;
    });

    Ok(Value::Null)
}

pub fn mpl_showimage(args: Vec<Value>) -> Result<Value> {
    // Check for unexpected arguments
    if !args.is_empty() {
        return Err(format!("mpl_showimage() expects 0 arguments, got {}", args.len()));
    }

    // Step 1: Save the current plot to "output.png"
    let filename = "output.png";
    mpl_savefig(vec![Value::String(filename.to_string())])?;

    // Step 2: Load the saved image
    let img = ImageReader::open(Path::new(filename))
        .map_err(|e| format!("Failed to open image '{}': {}", filename, e))?
        .decode()
        .map_err(|e| format!("Failed to decode image '{}': {}", filename, e))?
        .into_rgba8();
    let (width, height) = img.dimensions();

    // Step 3: Convert the image to a pixel buffer for minifb
    let buffer: Vec<u32> = img
        .pixels()
        .map(|p| {
            let r = p[0] as u32;
            let g = p[1] as u32;
            let b = p[2] as u32;
            let a = p[3] as u32;
            (a << 24) | (r << 16) | (g << 8) | b
        })
        .collect();

    // Step 4: Create a minifb window
    let mut window = Window::new(
        "Plot Display",
        width as usize,
        height as usize,
        WindowOptions::default()
    ).map_err(|e| format!("Failed to create window: {}", e))?;

    // Limit updates to ~60 FPS
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // Step 5: Display the image in a loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .map_err(|e| format!("Failed to update window: {}", e))?;
    }

    // Step 6: Clear the current figure after the window closes
    CURRENT_FIGURE.with(|cf| {
        let mut guard = cf.lock().unwrap();
        *guard = None;
    });

    Ok(Value::Null)
}

/// --- Helper Functions ---

/// Extract an array of numbers from a Value::Array.
fn extract_number_array(val: &Value, context: &str) -> Result<Vec<f64>> {
    match val {
        Value::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                let locked = item.lock().unwrap();
                match &*locked {
                    Value::Number(n) => vec.push(*n),
                    _ => {
                        return Err(format!("{}: Expected number in array", context));
                    }
                }
            }
            Ok(vec)
        }
        _ => Err(format!("{}: Expected an array", context)),
    }
}

/// Extract an array of strings from a Value::Array.
fn extract_string_array(val: &Value, context: &str) -> Result<Vec<String>> {
    match val {
        Value::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                let locked = item.lock().unwrap();
                match &*locked {
                    Value::String(s) => vec.push(s.clone()),
                    _ => {
                        return Err(format!("{}: Expected string in array", context));
                    }
                }
            }
            Ok(vec)
        }
        _ => Err(format!("{}: Expected an array", context)),
    }
}

/// Extract a string from a Value.
fn extract_string(val: &Value, context: &str) -> Result<String> {
    match val {
        Value::String(s) => Ok(s.clone()),
        _ => Err(format!("{}: Expected a string", context)),
    }
}

/// Extract a 2-element range from a Value::Array.
fn extract_range(val: &Value, context: &str) -> Result<(f64, f64)> {
    match val {
        Value::Array(arr) if arr.len() == 2 => {
            let a = {
                let locked = arr[0].lock().unwrap();
                match &*locked {
                    Value::Number(n) => *n,
                    _ => {
                        return Err(format!("{}: Expected number for range", context));
                    }
                }
            };
            let b = {
                let locked = arr[1].lock().unwrap();
                match &*locked {
                    Value::Number(n) => *n,
                    _ => {
                        return Err(format!("{}: Expected number for range", context));
                    }
                }
            };
            Ok((a, b))
        }
        _ => Err(format!("{}: Expected an array of 2 numbers", context)),
    }
}

/// Extract a 2D array (matrix) of numbers.
fn extract_number_matrix(val: &Value, context: &str) -> Result<Vec<Vec<f64>>> {
    match val {
        Value::Array(arr) => {
            let mut matrix = Vec::new();
            for row in arr {
                let locked = row.lock().unwrap();
                let r = extract_number_array(&locked, context)?;
                matrix.push(r);
            }
            Ok(matrix)
        }
        _ => Err(format!("{}: Expected an array of arrays", context)),
    }
}
