use macroquad::prelude::*;

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
    texture: Texture2D,
    hitbox: Hitbox,
}

impl SpeedPortal {
    async fn new(x: f32, y: f32, speed_change: f32) -> SpeedPortal {
        let texture = load_texture("assets/speedportal.png").await.unwrap();
        texture.set_filter(FilterMode::Nearest);
        SpeedPortal {
            used: false,
            speed_change,
            texture,
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

    fn draw(&self) {
        draw_texture_ex(&self.texture, self.hitbox.x, self.hitbox.y, WHITE, DrawTextureParams {
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
struct Player {
    x: f32,
    y: f32,
    vy: f32,
    is_facing_up: bool,
    x_speed_mult: f32,
    texture: Texture2D,
    hitbox: Hitbox,
}

impl Player {
    async fn new(x: f32, y: f32) -> Player {
        let texture = load_texture("assets/player.png").await.unwrap();
        texture.set_filter(FilterMode::Nearest);
        Player {
            x,
            y,
            vy: 0.0,
            is_facing_up: true, // Start out flying because the click from the titlescreen persists
            x_speed_mult: 3.5,
            texture,
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

    fn draw(&self) {
        draw_texture_ex(&self.texture, self.x - self.hitbox.width / 1.5 + self.hitbox.width / 2., self.y - self.hitbox.height / 1.5 + self.hitbox.height / 2., WHITE, DrawTextureParams { // One line of goddamn code
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
    async fn new() -> World {
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
                SpeedPortal::new(1075.0, -200.0, 2.0).await,
            ],
        }
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

    async fn draw(&self) {
        let wall = load_texture("assets/wall.png").await.unwrap();
        wall.set_filter(FilterMode::Nearest);
        let movingplatform = load_texture("assets/movingplatform.png").await.unwrap();
        movingplatform.set_filter(FilterMode::Nearest);
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
            object.draw();
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
    async fn new() -> TitleScreen {
        // All buttons are stored inside assets/buttons.png. The first button is at (0, 0) and is
        // 128x32 pixels. The second button is at (0, 32) and is also 128x32 pixels. etc. etc. etc.
        let new_game_button = Button::new(100.0, 300.0, get_texture_from_spritesheet(
            &load_texture("assets/buttons.png").await.unwrap(),
            0, 0, 128, 32,
        ), get_texture_from_spritesheet(
            &load_texture("assets/buttons.png").await.unwrap(),
            0, 32, 128, 32,
        ), "new_game".to_owned());
        let statistics_button = Button::new(100.0, 370.0, get_texture_from_spritesheet(
            &load_texture("assets/buttons.png").await.unwrap(),
            0, 64, 128, 32,
        ), get_texture_from_spritesheet(
            &load_texture("assets/buttons.png").await.unwrap(),
            0, 96, 128, 32,
        ), "statistics".to_owned());
        let settings_button = Button::new(100.0, 440.0, get_texture_from_spritesheet(
            &load_texture("assets/buttons.png").await.unwrap(),
            0, 128, 128, 32,
        ), get_texture_from_spritesheet(
            &load_texture("assets/buttons.png").await.unwrap(),
            0, 160, 128, 32,
        ), "settings".to_owned());

        let leader_board_button = MiniButton::new(360.0, 440.0, get_texture_from_spritesheet(
            &load_texture("assets/minibuttons.png").await.unwrap(),
            0, 0, 32, 32,
        ), get_texture_from_spritesheet(
            &load_texture("assets/minibuttons.png").await.unwrap(),
            0, 32, 32, 32,
        ), "leader_board".to_owned());
        TitleScreen {
            title: "Hardest Game Ever v1.0.0".to_owned(),
            buttons: vec![
                new_game_button,
                statistics_button,
                settings_button,
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

    async fn draw(&self) {
        draw_text(&self.title, 100.0, 200.0, 96.0, WHITE);
        for button in &self.buttons {
            button.draw();
        }
        for button in &self.mini_buttons {
            button.draw();
        }
        let player = load_texture("assets/player.png").await.unwrap();
        player.set_filter(FilterMode::Nearest);
        draw_texture_ex(&player, 1000.0 - (get_time().sin() * 16.0) as f32, 500.0 - (get_time().sin() * 16.0) as f32, WHITE, DrawTextureParams {
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
    let mut title_screen = TitleScreen::new().await;

    loop {
        set_default_camera();
        clear_background(BLACK);

        let next_screen = title_screen.update();
        title_screen.draw().await;

        if next_screen == "new_game" {
            game().await;
        } else if next_screen == "statistics" {
            // statistics().await;
        } else if next_screen == "settings" {
            // settings().await;
        }

        next_frame().await;
    }
}

async fn game() {
    let mut player = Player::new(0.0, 0.0).await;

    let mut world = World::new().await;

    let mut cam = Camera2D {
        zoom: vec2(1.0 / WINDOW_WIDTH * 2.0, 1.0 / WINDOW_HEIGHT * 2.0),
        ..Default::default()
    };

    let mut attempts = 0;

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
            player = Player::new(0.0, 0.0).await;
            for object in &mut world.speed_increases {
                object.used = false;
            }
            attempts += 1;
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

        player.draw();
        world.draw().await;

        next_frame().await
    }
}
