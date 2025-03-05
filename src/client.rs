use std::env;
use serde_json::json;

use macroquad::prelude::*;
use minreq::{ get, post };

mod router;

fn get_image(path: &str) -> Texture2D {
    let response = get(format!("https://muhtasim-rasheed.github.io/hardest-game-ever/assets/{}", path))
        .send()
        .unwrap();
    if response.status_code == 200 {
        let image = Texture2D::from_file_with_format(
            &response.as_bytes(),
            Some(ImageFormat::Png),
        );
        image.set_filter(FilterMode::Nearest);
        return image;
    } else {
        panic!("Failed to load image as response code was not 200 ({}).", response.status_code);
    }
}

fn submit_score(score: i32) {
    // Get the username from the PC
    let username = env::var("USERNAME").unwrap_or("Player".to_owned());

    let request = post("https://hardest-game-ever-d2ht.shuttle.app/submit")
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&router::Score {
            player: username,
            score: score as u32,
        }).unwrap())
        .send()
        .unwrap();

    if request.status_code != 200 {
        panic!("Failed to submit score as response code was not 200 ({}).", request.status_code);
    }
}

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;

const PLAYER_SPEED: f32 = 1.0;

fn round(x: f32, places: i32) -> f32 {
    let factor = 10.0f32.powi(places);
    (x * factor).round() / factor
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn line_intersect(p1: Vec2, p2: Vec2, q1: Vec2, q2: Vec2) -> bool {
    let r = p2 - p1;
    let s = q2 - q1;

    let r_cross_s = cross(r, s);
    let q_minus_p = q1 - p1;

    if r_cross_s == 0.0 {
        return false;
    }

    let t = cross(q_minus_p, s) / r_cross_s;
    let u = cross(q_minus_p, r) / r_cross_s;

    t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];

        if ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn draw_texture_across_hitbox(texture: &Texture2D, hitbox: &Hitbox) {
    let texture_width = texture.width();
    let texture_height = texture.height();

    let cols = (hitbox.width / (texture_width * 2.0)).ceil() as i32;
    let rows = (hitbox.height / (texture_height * 2.0)).ceil() as i32;

    let texture_width = hitbox.width / cols as f32;
    let texture_height = hitbox.height / rows as f32;

    for i in 0..cols {
        for j in 0..rows {
            let x = hitbox.x + i as f32 * texture_width;
            let y = hitbox.y + j as f32 * texture_height;

            draw_texture_ex(texture, x, y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(texture_width, texture_height)),
                ..Default::default()
            });
        }
    }
}

fn draw_texture_across_polygon(texture: &Texture2D, polygon: &PolygonHitbox) {
    // Compute the bounding box of the polygon
    let min_x = polygon.points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = polygon.points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let min_y = polygon.points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = polygon.points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

    let texture_width = texture.width();
    let texture_height = texture.height();

    let cols = (((max_x - min_x) / texture_width) / 2.).ceil() as i32;
    let rows = (((max_y - min_y) / texture_height) / 2.).ceil() as i32;

    let texture_width = (max_x - min_x) / cols as f32;
    let texture_height = (max_y - min_y) / rows as f32;

    for i in 0..cols {
        for j in 0..rows {
            let x = min_x + i as f32 * texture_width;
            let y = min_y + j as f32 * texture_height;
            let center = vec2(x + texture_width / 2.0, y + texture_height / 2.0);

            // Only draw if the tile's center is inside the polygon
            if point_in_polygon(center, &polygon.points) {
                // draw_texture(texture, x, y, WHITE);
                draw_texture_ex(texture, x, y, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(texture_width, texture_height)),
                    ..Default::default()
                });
            }
        }
    }
}

fn get_texture_from_spritesheet(
    spritesheet: &Texture2D,
    sprite_x: i32,
    sprite_y: i32,
    sprite_width: i32,
    sprite_height: i32,
) -> Texture2D {
    let image = spritesheet.get_texture_data();

    let mut new_image = Image::gen_image_color(sprite_width as u16, sprite_height as u16, WHITE);

    let spritesheet_width = image.width() as i32;
    let spritesheet_height = image.height() as i32;

    for y in 0..sprite_height {
        for x in 0..sprite_width {
            let src_x = sprite_x + x;
            let src_y = sprite_y + y;

            // Ensure we are within the image bounds
            if src_x < spritesheet_width && src_y < spritesheet_height {
                let color = image.get_pixel(src_x as u32, src_y as u32);
                new_image.set_pixel(x as u32, y as u32, color);
            }
        }
    }

    Texture2D::from_image(&new_image)
}

struct Hitbox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
}

impl Hitbox {
    fn new(x: f32, y: f32, width: f32, height: f32, color: Color) -> Hitbox {
        Hitbox { x, y, width, height, color }
    }

    fn collides(&self, other: &Hitbox) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    fn get_pos(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    fn draw(&self) {
        draw_rectangle_lines(self.x, self.y, self.width, self.height, 5.0, self.color);
    }
}

struct PolygonHitbox {
    points: Vec<Vec2>,
    color: Color,
}

impl PolygonHitbox {
    fn new(points: Vec<Vec2>, color: Color) -> PolygonHitbox {
        PolygonHitbox { points, color }
    }

    fn collides(&self, other: &PolygonHitbox) -> bool {
        for i in 0..self.points.len() {
            let p1 = self.points[i];
            let p2 = self.points[(i + 1) % self.points.len()];

            for j in 0..other.points.len() {
                let q1 = other.points[j];
                let q2 = other.points[(j + 1) % other.points.len()];

                if line_intersect(p1, p2, q1, q2) {
                    return true;
                }
            }
        }
        false
    }

    fn draw(&self) {
        for i in 0..self.points.len() {
            let p1 = self.points[i];
            let p2 = self.points[(i + 1) % self.points.len()];
            draw_line(p1.x, p1.y, p2.x, p2.y, 3.0, self.color);
        }
    }
}

struct MovingObject {
    from: Vec2,
    to: Vec2,
    forward: bool,
    speed: f32,
    hitbox: Hitbox,
}

impl MovingObject {
    fn new(from: Vec2, to: Vec2, width: f32, height: f32, speed: f32) -> MovingObject {
        MovingObject {
            from,
            to,
            forward: true,
            speed,
            hitbox: Hitbox::new(from.x, from.y, width, height, ORANGE),
        }
    }
   
    fn update(&mut self) {
        let direction = self.to - self.from;
        let distance = direction.length();
        let normalized_direction = direction / distance;

        if self.forward {
            self.hitbox.x += normalized_direction.x * self.speed;
            self.hitbox.y += normalized_direction.y * self.speed;
        } else {
            self.hitbox.x -= normalized_direction.x * self.speed;
            self.hitbox.y -= normalized_direction.y * self.speed;
        }

        let current_distance = (self.hitbox.get_pos() - self.from).length();
        
        if current_distance >= distance {
            self.forward = false;
        } else if current_distance <= 0.0 {
            self.forward = true;
        }
    }

    fn draw(&self) {
        self.hitbox.draw();
    }
}

struct SpeedPortal {
    used: bool,
    speed_change: f32,
    hitbox: Hitbox,
}

impl SpeedPortal {
    fn new(x: f32, y: f32, speed_change: f32) -> SpeedPortal {
        SpeedPortal {
            used: false,
            speed_change,
            hitbox: Hitbox::new(x, y, 64., 128., PURPLE),
        }
    }

    fn update(&mut self, player: &mut Player) {
        if player.hitbox.collides(&self.hitbox) {
            if self.used {
                return;
            }
            player.x_speed_mult *= self.speed_change;
            self.used = true;
        }
    }

    fn draw(&self, texture: &Texture2D) {
        draw_texture_ex(&texture, self.hitbox.x, self.hitbox.y, WHITE, DrawTextureParams {
            dest_size: Some(vec2(self.hitbox.width, self.hitbox.height)),
            ..Default::default()
        });
    }
}

impl Clone for Hitbox {
    fn clone(&self) -> Hitbox {
        Hitbox {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            color: self.color,
        }
    }
}

impl Clone for PolygonHitbox {
    fn clone(&self) -> PolygonHitbox {
        PolygonHitbox {
            points: self.points.clone(),
            color: self.color,
        }
    }
}

impl Clone for MovingObject {
    fn clone(&self) -> MovingObject {
        MovingObject {
            from: self.from,
            to: self.to,
            forward: self.forward,
            speed: self.speed,
            hitbox: self.hitbox.clone(),
        }
    }
}

enum Objects {
    Hitbox(Hitbox),
    PolygonHitbox(PolygonHitbox),
    MovingObject(MovingObject),
    SpeedPortal(SpeedPortal),
}

struct Player {
    x: f32,
    y: f32,
    vy: f32,
    is_facing_up: bool,
    x_speed_mult: f32,
    hitbox: Hitbox,
}

impl Player {
    fn new(x: f32, y: f32) -> Player {
        // let texture = load_texture("assets/player.png").await.unwrap();
        // texture.set_filter(FilterMode::Nearest);
        Player {
            x,
            y,
            vy: 0.0,
            is_facing_up: true, // Start out flying because the click from the titlescreen persists
            x_speed_mult: 3.5,
            hitbox: Hitbox::new(x, y, 32.0, 24.0, RED),
        }
    }
   
    fn update(&mut self, world: &World) -> bool {
        if world.player_hit_check(self) {
            return true; // Player dies
        }

        // Movement logic only runs if no collision
        if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space) {
            self.is_facing_up = !self.is_facing_up;
        }

        if self.is_facing_up {
            self.vy -= PLAYER_SPEED;
        } else {
            self.vy += PLAYER_SPEED;
        }

        self.y += self.vy;
        self.hitbox.y = self.y;

        self.x += PLAYER_SPEED * self.x_speed_mult;
        self.hitbox.x = self.x;

        self.vy *= 0.9;

        false // Player survives
    }

    fn draw(&self, texture: &Texture2D) {
        draw_texture_ex(&texture, self.x - self.hitbox.width / 1.5 + self.hitbox.width / 2., self.y - self.hitbox.height / 1.5 + self.hitbox.height / 2., WHITE, DrawTextureParams { // One line of goddamn code
            dest_size: Some(vec2(self.hitbox.width * 1.5, self.hitbox.height * 1.5)),
            rotation: ((self.vy * 20.0) / (PLAYER_SPEED * self.x_speed_mult)).to_radians(),
            ..Default::default()
        });
    }
}

struct World {
    objects: Vec<Hitbox>,
    poly_objects: Vec<PolygonHitbox>,
    moving_objects: Vec<MovingObject>,
    speed_increases: Vec<SpeedPortal>,
}

impl World {
    fn new() -> World {
        World {
            objects: vec![
                Hitbox::new(0.0, 250.0, 5000.0, 50.0, GREEN),
                Hitbox::new(0.0, -300.0, 5000.0, 50.0, GREEN),

                Hitbox::new(500.0, 15.0, 50.0, 235.0, GREEN),
                Hitbox::new(625.0, -250.0, 50.0, 235.0, GREEN),
            ],
            poly_objects: vec![
                PolygonHitbox::new(vec![
                    vec2(775.0, -80.0), // Top left
                    vec2(775.0, 250.0), // Bottom left
                    vec2(1075.0, 250.0), // Bottom right
                    vec2(1075.0, 0.0), // Top right (1)
                    vec2(975.0, -80.0), // Top right (2)
                ], GREEN),
            ],
            moving_objects: vec![
                MovingObject::new(vec2(1100.0, -225.0), vec2(1200.0, 150.0), 50.0, 100.0, 3.0),
                MovingObject::new(vec2(1300.0, 150.0), vec2(1400.0, -225.0), 50.0, 100.0, 3.0),
                MovingObject::new(vec2(1500.0, 0.0), vec2(1600.0, 0.0), 100.0, 50.0, 3.0),
            ],
            speed_increases: vec![
                SpeedPortal::new(1075.0, -200.0, 2.0),
            ],
        }
    }

    fn from_json(data: &str) -> World {
        let world: serde_json::Value = serde_json::from_str(data).unwrap();
        let mut objects = Vec::new();
        let mut poly_objects = Vec::new();
        let mut moving_objects = Vec::new();
        let mut speed_increases = Vec::new();
        for object in world["objects"].as_array().unwrap() {
            objects.push(Hitbox::new(
                object["x"].as_f64().unwrap() as f32,
                object["y"].as_f64().unwrap() as f32,
                object["width"].as_f64().unwrap() as f32,
                object["height"].as_f64().unwrap() as f32,
                GREEN,
            ));
        }
        for object in world["poly_objects"].as_array().unwrap() {
            let mut points = Vec::new();
            for point in object["points"].as_array().unwrap() {
                points.push(vec2(
                    point["x"].as_f64().unwrap() as f32,
                    point["y"].as_f64().unwrap() as f32,
                ));
            }
            poly_objects.push(PolygonHitbox::new(points, GREEN));
        }
        for object in world["moving_objects"].as_array().unwrap() {
            moving_objects.push(MovingObject::new(
                vec2(
                    object["from"]["x"].as_f64().unwrap() as f32,
                    object["from"]["y"].as_f64().unwrap() as f32,
                ),
                vec2(
                    object["to"]["x"].as_f64().unwrap() as f32,
                    object["to"]["y"].as_f64().unwrap() as f32,
                ),
                object["width"].as_f64().unwrap() as f32,
                object["height"].as_f64().unwrap() as f32,
                object["speed"].as_f64().unwrap() as f32,
            ));
        }
        for object in world["speed_increases"].as_array().unwrap() {
            speed_increases.push(SpeedPortal::new(
                object["x"].as_f64().unwrap() as f32,
                object["y"].as_f64().unwrap() as f32,
                object["speed_change"].as_f64().unwrap() as f32,
            ));
        }
        World {
            objects,
            poly_objects,
            moving_objects,
            speed_increases,
        }
    }

    fn as_json(&self) -> String {
        let mut objects = Vec::new();
        let mut poly_objects = Vec::new();
        let mut moving_objects = Vec::new();
        let mut speed_increases = Vec::new();
        for object in &self.objects {
            objects.push(json!({
                "x": object.x,
                "y": object.y,
                "width": object.width,
                "height": object.height,
            }));
        }
        for object in &self.poly_objects {
            let mut points = Vec::new();
            for point in &object.points {
                points.push(json!({
                    "x": point.x,
                    "y": point.y,
                }));
            }
            poly_objects.push(json!({
                "points": points,
            }));
        }
        for object in &self.moving_objects {
            moving_objects.push(json!({
                "from": {
                    "x": object.from.x,
                    "y": object.from.y,
                },
                "to": {
                    "x": object.to.x,
                    "y": object.to.y,
                },
                "width": object.hitbox.width,
                "height": object.hitbox.height,
                "speed": object.speed,
            }));
        }
        for object in &self.speed_increases {
            speed_increases.push(json!({
                "x": object.hitbox.x,
                "y": object.hitbox.y,
                "speed_change": object.speed_change,
            }));
        }
        json!({
            "objects": objects,
            "poly_objects": poly_objects,
            "moving_objects": moving_objects,
            "speed_increases": speed_increases,
        }).to_string()
    }

    fn player_hit_check(&self, player: &Player) -> bool {
        for object in &self.objects {
            if player.hitbox.collides(object) {
                return true;
            }
        }
        for object in &self.poly_objects {
            if object.collides(&PolygonHitbox::new(vec![
                vec2(player.x, player.y),
                vec2(player.x + player.hitbox.width, player.y),
                vec2(player.x + player.hitbox.width, player.y + player.hitbox.height),
                vec2(player.x, player.y + player.hitbox.height),
            ], RED)) {
                return true;
            }
        }
        for object in &self.moving_objects {
            if player.hitbox.collides(&object.hitbox) {
                return true;
            }
        }
        false
    }

    fn update(&mut self, player: &mut Player) {
        for object in &mut self.moving_objects {
            object.update();
        }

        for object in &mut self.speed_increases {
            object.update(player);
        }
    }

    async fn draw(&self, wall: &Texture2D, movingplatform: &Texture2D, speedportal: &Texture2D) {
        for object in &self.objects {
            // object.draw();
            draw_texture_across_hitbox(&wall, object);
        }
        for object in &self.poly_objects {
            // object.draw();
            draw_texture_across_polygon(&wall, object);
        }
        for object in &self.moving_objects {
            // object.draw();
            draw_texture_across_hitbox(&movingplatform, &object.hitbox);
        }
        for object in &self.speed_increases {
            object.draw(&speedportal);
        }
    }
}

// Man, I forgot to make the title screen. Well, let's do it now.
struct Button {
    x: f32,
    y: f32,
    hitbox: Hitbox,
    texture: Texture2D,
    hover_texture: Texture2D,
    is_hovered: bool,
    id: String,
}

impl Button {
    fn new(x: f32, y: f32, texture: Texture2D, hover_texture: Texture2D, id: String) -> Button {
        texture.set_filter(FilterMode::Nearest);
        hover_texture.set_filter(FilterMode::Nearest);
        Button {
            x,
            y,
            hitbox: Hitbox::new(x, y, 256.0, 64.0, BLUE),
            texture,
            hover_texture,
            is_hovered: false,
            id,
        }
    }

    fn update(&mut self) -> bool {
        let mouse_pos = mouse_position();
        if self.hitbox.collides(&Hitbox::new(mouse_pos.0, mouse_pos.1, 1.0, 1.0, BLUE)) {
            self.is_hovered = true;
        } else {
            self.is_hovered = false;
        }

        if self.is_hovered {
            if is_mouse_button_pressed(MouseButton::Left) {
                return true;
            }
        }
        false
    }

    fn draw(&self) {
        if self.is_hovered {
            draw_texture_ex(&self.hover_texture, self.x, self.y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(self.hitbox.width, self.hitbox.height)),
                ..Default::default()
            });
        } else {
            draw_texture_ex(&self.texture, self.x, self.y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(self.hitbox.width, self.hitbox.height)),
                ..Default::default()
            });
        }
    }
}

struct MiniButton {
    x: f32,
    y: f32,
    hitbox: Hitbox,
    texture: Texture2D,
    hover_texture: Texture2D,
    is_hovered: bool,
    id: String,
}

impl MiniButton {
    fn new(x: f32, y: f32, texture: Texture2D, hover_texture: Texture2D, id: String) -> MiniButton {
        texture.set_filter(FilterMode::Nearest);
        hover_texture.set_filter(FilterMode::Nearest);
        MiniButton {
            x,
            y,
            hitbox: Hitbox::new(x, y, 64.0, 64.0, BLUE),
            texture,
            hover_texture,
            is_hovered: false,
            id,
        }
    }

    fn update(&mut self) -> bool {
        let mouse_pos = mouse_position();
        if self.hitbox.collides(&Hitbox::new(mouse_pos.0, mouse_pos.1, 1.0, 1.0, BLUE)) {
            self.is_hovered = true;
        } else {
            self.is_hovered = false;
        }

        if self.is_hovered {
            if is_mouse_button_pressed(MouseButton::Left) {
                return true;
            }
        }
        false
    }

    fn draw(&self) {
        if self.is_hovered {
            draw_texture_ex(&self.hover_texture, self.x, self.y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(self.hitbox.width, self.hitbox.height)),
                ..Default::default()
            });
        } else {
            draw_texture_ex(&self.texture, self.x, self.y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(self.hitbox.width, self.hitbox.height)),
                ..Default::default()
            });
        }
    }
}

struct TitleScreen {
    title: String,
    buttons: Vec<Button>,
    mini_buttons: Vec<MiniButton>,
}

impl TitleScreen {
    fn new(buttons_texture: &Texture2D, minibuttons_texture: &Texture2D) -> TitleScreen {
        // All buttons are stored inside assets/buttons.png. The first button is at (0, 0) and is
        // 128x32 pixels. The second button is at (0, 32) and is also 128x32 pixels. etc. etc. etc.
        let new_game_button = Button::new(100.0, 300.0, get_texture_from_spritesheet(
            &buttons_texture,
            0, 0, 128, 32,
        ), get_texture_from_spritesheet(
            &buttons_texture,
            0, 32, 128, 32,
        ), "new_game".to_owned());
        let statistics_button = Button::new(100.0, 370.0, get_texture_from_spritesheet(
            &buttons_texture,
            0, 64, 128, 32,
        ), get_texture_from_spritesheet(
            &buttons_texture,
            0, 96, 128, 32,
        ), "statistics".to_owned());
        // let settings_button = Button::new(100.0, 440.0, get_texture_from_spritesheet(
        //     &buttons_texture,
        //     0, 128, 128, 32,
        // ), get_texture_from_spritesheet(
        //     &buttons_texture,
        //     0, 160, 128, 32,
        // ), "settings".to_owned());

        let leader_board_button = MiniButton::new(360.0, 300.0, get_texture_from_spritesheet(
            &minibuttons_texture,
            0, 0, 32, 32,
        ), get_texture_from_spritesheet(
            &minibuttons_texture,
            0, 32, 32, 32,
        ), "leader_board".to_owned());
        TitleScreen {
            title: "Hardest Game Ever v1.5.1".to_owned(),
            buttons: vec![
                new_game_button,
                statistics_button,
                // settings_button,
            ],
            mini_buttons: vec![
                leader_board_button,
            ],
        }
    }

    fn update(&mut self) -> String {
        for button in &mut self.buttons {
            if button.update() {
                return button.id.clone();
            }
        }
        for button in &mut self.mini_buttons {
            if button.update() {
                return button.id.clone();
            }
        }
        "continue".to_owned()
    }

    fn draw(&self, player_texture: &Texture2D) {
        draw_text(&self.title, 100.0, 200.0, 96.0, WHITE);
        for button in &self.buttons {
            button.draw();
        }
        for button in &self.mini_buttons {
            button.draw();
        }
        draw_texture_ex(&player_texture, 1000.0 - (get_time().sin() * 16.0) as f32, 500.0 - (get_time().sin() * 16.0) as f32, WHITE, DrawTextureParams {
            dest_size: Some(vec2((192.0 + get_time().sin() * 32.0) as f32, (144.0 + get_time().sin() * 32.0) as f32)),
            rotation: ((get_time() * 2.0).sin() * 0.5) as f32,
            ..Default::default()
        });
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Hardest Game Ever".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // ALL THE TEXTURES
    let player_texture = get_image("player.png");
    let wall_texture = get_image("wall.png");
    let movingplatform_texture = get_image("movingplatform.png");
    let speedportal_texture = get_image("speedportal.png");
    let buttons_texture = get_image("buttons.png");
    let minibuttons_texture = get_image("minibuttons.png");

    let leaderboard_res = get("https://hardest-game-ever-d2ht.shuttle.app/leaderboard")
        .send()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let world_res = get("https://hardest-game-ever-d2ht.shuttle.app/world")
        .send()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let mut title_screen = TitleScreen::new(&buttons_texture, &minibuttons_texture);

    let mut best_score = 0;

    loop {
        set_default_camera();
        clear_background(BLACK);

        let next_screen = title_screen.update();
        title_screen.draw(&player_texture);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if next_screen == "new_game" {
            best_score = game(world_res.clone(), player_texture.clone(), wall_texture.clone(), movingplatform_texture.clone(), speedportal_texture.clone()).await;
            // Submit the score to the server on a separate thread
            std::thread::spawn(move || {
                submit_score(best_score as i32);
            });
        } else if next_screen == "statistics" {
            // statistics().await;
            // Show best score if its higher than leaderboard score
            let mut leaderboard_selfbest = 0;
            let selfname = env::var("USERNAME").unwrap_or("Player".to_owned());
            for score in serde_json::from_str::<Vec<router::Score>>(&leaderboard_res.as_str()).unwrap() {
                if score.player == selfname {
                    leaderboard_selfbest = score.score;
                    break;
                }
            }
            let score = if best_score > leaderboard_selfbest {
                best_score
            } else {
                leaderboard_selfbest
            };
            statistics(score).await;
        } else if next_screen == "leader_board" {
            leaderboard(leaderboard_res.clone()).await;
        }

        next_frame().await;
    }
}

async fn statistics(leaderboard_selfbest: u32) {
    loop {
        set_default_camera();
        clear_background(BLACK);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        draw_text("Statistics", 100.0, 100.0, 48.0, WHITE);
        draw_text(&format!("Your best score: {}s", round(leaderboard_selfbest as f32 / 60.0, 2)), 100.0, 200.0, 36.0, GOLD);

        next_frame().await;
    }
}

async fn leaderboard(response: String) {
    let scores: Vec<router::Score>;

    scores = serde_json::from_str(&response.as_str()).unwrap();

    loop {
        set_default_camera();
        clear_background(BLACK);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        for i in 0..scores.len() {
            let color = if i == 0 {
                GOLD
            } else if i == 1 {
                GRAY
            } else if i == 2 {
                BROWN
            } else {
                WHITE
            };
            draw_text(&format!("{}: {} - {}s", i + 1, scores[i].player, round(scores[i].score as f32 / 60.0, 2)), 100.0, 100.0 + i as f32 * 50.0, 36.0 + (10.0 - i as f32) * 4.0, color);
        }

        draw_text("Leaderboard may not be up to date, restart the game to refresh", 100.0, 800.0, 24.0, GRAY);

        next_frame().await;
    }
}

async fn game(world_res: String, player_texture: Texture2D, wall_texture: Texture2D, movingplatform_texture: Texture2D, speedportal_texture: Texture2D) -> u32 {
    let mut player = Player::new(0.0, 0.0);

    let mut world = World::from_json(&world_res.as_str());

    println!("{}", world.as_json());

    let mut cam = Camera2D {
        zoom: vec2(1.0 / WINDOW_WIDTH * 2.0, 1.0 / WINDOW_HEIGHT * 2.0),
        ..Default::default()
    };

    let mut attempts = 0;
    let mut score = 0;
    let mut best_score = 0;

    let mut bg_color = BLACK;
    
    loop {
        set_camera(&cam);
        clear_background(bg_color);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if !(bg_color == BLACK) {
            bg_color.r /= 1.015;
            bg_color.g /= 1.015;
            bg_color.b /= 1.015;
            bg_color.r = round(bg_color.r, 4);
            bg_color.g = round(bg_color.g, 4);
            bg_color.b = round(bg_color.b, 4);
        }
        
        if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space) {
            bg_color = Color::new(0.125, 0.125, 0.25, 1.0);
        }

        let dead = player.update(&world);

        if dead {
            player = Player::new(0.0, 0.0);
            for object in &mut world.speed_increases {
                object.used = false;
            }
            attempts += 1;
            score = 0;
            next_frame().await;
        }
        
        world.update(&mut player);

        cam.target.x = player.x + 200.;

        draw_text_ex("Hardest Game Ever", 0.0, 0.0, TextParams {
            font_size: 48,
            rotation: 3.0f32.to_radians(),
            ..Default::default()
        });
        draw_text("Use the mouse or space bar to change direction", -50.0, 60.0, 24.0, WHITE);
        draw_text(&format!("Attempts: {}", attempts), 0.0, 100.0, 36.0, WHITE);

        player.draw(&player_texture);
        world.draw(&wall_texture, &movingplatform_texture, &speedportal_texture).await;

        set_default_camera();

        draw_text(&format!("Score: {}", round(score as f32 / 60.0, 2)), 10.0, 50.0, 30.0, WHITE);
        draw_text(&format!("Best Score: {}", round(best_score as f32 / 60.0, 2)), 10.0, 100.0, 30.0, WHITE);

        score += 1;

        if score > best_score {
            best_score = score;
        }

        next_frame().await
    }

    best_score
}
