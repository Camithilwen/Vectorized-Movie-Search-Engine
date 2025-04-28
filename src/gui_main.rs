use eframe::egui;

#[derive(Default)]
struct MyApp {
    query: String,
    query_keywords: String, 
    recommendations: Vec<MovieRecommendation>
}
struct MovieRecommendation{
    title: String, 
    release_year: String
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui|{ui.heading("Movie Recommendation Search");
            ui.label("Please enter either the movie title or key words describing the movie*.");
            ui.add_space(8.0);
            ui.label("*IMPORTANT NOTE: Please enter a movie title released between 1901-2017. Please enter key words for movies released outside of this range
            to ensure maximum possible accuracy.");
            ui.add_space(10.0);

            
            ui.vertical_centered(|ui|{ui.horizontal(|ui|{
                ui.vertical(|ui|{ui.label("movie title:");
                ui.text_edit_singleline(&mut self.query);
                ui.button("Search").clicked();
                })});
                ui.add_space(10.0);
                ui.vertical(|ui|{ui.label("key words:");
                ui.text_edit_singleline(&mut self.query);
                ui.button("Search").clicked();
                });
            });
            ui.add_space(10.0);
         
            ui.separator();

            ui.label("Recommendations:");

            ui.horizontal_centered(|ui|{
                for movie in &self.recommendations{
                    ui.vertical(|ui|{
                        ui.label(&movie.title);
                        ui.label(&format!("({})", movie.release_year));
                    });
                    ui.add_space(10.0);
                }
            })
        });
    });
    }
}

impl MyApp {
    fn perform_search(&mut self) {
        if self.query.is_empty() {
            self.recommendations.clear();
        } else {
            self.recommendations = vec![
                MovieRecommendation {
                    title,
                    release_year
                },
                MovieRecommendation {
                    title,
                    release_year
                },
                MovieRecommendation {
                    title,
                    release_year
                }
            ]
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Movie Recommendation System",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}
