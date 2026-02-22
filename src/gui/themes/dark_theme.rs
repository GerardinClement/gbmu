use eframe::egui::{Visuals, Color32, Stroke};

pub fn get_dark_theme_visual() -> Visuals {
    let mut visuals = Visuals::default();

    // Couleur de fond du panneau principal
    visuals.panel_fill = Color32::from_hex("#13161d").expect("invalid hex color");

    // Couleur de fond des zones de saisie (TextEdit, ComboBox...)
    visuals.extreme_bg_color = Color32::from_hex("#1e2130").expect("invalid hex");

    // Couleur très légèrement différente du fond, utilisée pour les bandes alternées dans les tables
    visuals.faint_bg_color = Color32::from_hex("#000002").expect("invalid hex");

    // Fond des boutons/widgets au repos
    visuals.widgets.inactive.weak_bg_fill = Color32::from_hex("#1c2030").expect("invalid hex");

    // Contour des boutons/widgets au repos
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_hex("#3d4166").expect("invalid hex"));

    // Couleur du texte et icônes des widgets au repos
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::WHITE);

    // Fond des boutons/widgets au survol
    visuals.widgets.hovered.weak_bg_fill = Color32::from_hex("#242840").expect("invalid hex");

    // Contour des boutons/widgets au survol
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, Color32::from_hex("#8b5cf6").expect("invalid hex"));

    // Fond des boutons/widgets au clic
    visuals.widgets.active.weak_bg_fill = Color32::from_hex("#2d324e").expect("invalid hex");

    // Contour des boutons/widgets au clic
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::from_hex("#a16fdf").expect("invalid hex"));

    // Surlignage de la sélection de texte
    visuals.selection.bg_fill = Color32::from_hex("#8b5cf6").expect("invalid hex");

    // Contour de la zone de texte quand elle a le focus
    visuals.selection.stroke = Stroke::new(2.0, Color32::from_hex("#9b59b6").expect("invalid hex"));

    visuals
}