pub mod demo;
use anyhow::*;
use demo::traits::*;
use raylib::prelude::*;

impl demo::VectorImpl for Vector2 {
    type T = f32;

    fn new(x: f32, y: f32) -> Self {
        Self::new(x, y)
    }

    fn x(v: &Self) -> f32 {
        v.x
    }
    
    fn y(v: &Self) -> f32 {
        v.y
    }
}

impl demo::ColorImpl for Color {
    fn from_hex(hex: u32) -> Self {
        Self::get_color(hex)
    }
}

fn main() -> Result<()> {
    let world = demo::World::load();
    let level = &world.entities_demo;
    let (mut rl, thread) = raylib::init()
        .size(level.pixel_size.x as i32 * 2, level.pixel_size.y as i32 * 2)
        .title("Hello, World")
        .build();
    let mut tilesets = std::collections::HashMap::new();
    for tileset in [demo::ICONS, demo::INCA_FRONT, demo::INCA_BACK] {
        tilesets.insert(
            tileset.id,
            rl.load_texture(
                &thread,
                std::path::Path::new("src")
                    .join(tileset.path)
                    .to_str()
                    .context("Failed to convert tileset path to string!")?,
            )
            .map_err(|err| anyhow!(err))?,
        );
    }

    for entity in level.triggerables.entities() {
        if let demo::Entity::TriggerArea(trigger) = &entity.entity {
            for entity in &trigger.on_trigger {
                let entity = entity.find(&world).unwrap();
                if let demo::Entity::MessagePopUp(msg) = &entity.entity {
                    println!("{}", msg.text);
                }
            }
        }
    }

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(level.bg_color);
        let mut d = d.begin_mode2D(Camera2D {
            offset: Vector2::zero(),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 2.0,
        });

        for (pos, tile) in level
            .tiles
            .rect(Vector2::zero(), level.tiles.size())
            .filter_map(|(pos, tile)| tile.map(|tile| (pos, tile)))
        {
            d.draw_texture_rec(
                tilesets.get(&demo::Tiles::TILESET_ID).unwrap(),
                rrect(
                    tile.position.x * level.collisions.grid_size().x,
                    tile.position.y * level.collisions.grid_size().y,
                    level.collisions.grid_size().x
                        * if tile.flip.horizontal() { -1.0 } else { 1.0 },
                    level.collisions.grid_size().y * if tile.flip.vertical() { -1.0 } else { 1.0 },
                ),
                pos * level.collisions.grid_size(),
                Color::WHITE,
            );
        }

        for (pos, tiles) in level
            .collisions
            .autotile_rect(Vector2::zero(), level.collisions.size)
        {
            for tile in tiles {
                d.draw_texture_rec(
                    tilesets.get(&demo::Collisions::TILESET_ID).unwrap(),
                    rrect(
                        tile.position.x * level.collisions.grid_size().x,
                        tile.position.y * level.collisions.grid_size().y,
                        level.collisions.grid_size().x
                            * if tile.flip.horizontal() { -1.0 } else { 1.0 },
                        level.collisions.grid_size().y
                            * if tile.flip.vertical() { -1.0 } else { 1.0 },
                    ),
                    pos * level.collisions.grid_size(),
                    Color::WHITE,
                );
            }
        }
        for entity in level.triggerables.entities() {
            d.draw_rectangle_v(entity.top_left(), entity.size, Color::new(0, 255, 0, 30));
        }
        for entity in level.game_entities.entities() {
            if let demo::Entity::Enemy(enemy) = &entity.entity {
                let tile = enemy.enemy_type.icon().unwrap();
                d.draw_texture_pro(
                    tilesets.get(&demo::MonsterType::TILESET_ID).unwrap(),
                    rrect(tile.x * 16.0, tile.y * 16.0, 16.0, 16.0),
                    rrect(
                        entity.top_left().x,
                        entity.top_left().y,
                        entity.size.x,
                        entity.size.y,
                    ),
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                );
            } else if let demo::RenderMode::Tile {
                tileset,
                tile,
                size,
            } = entity.entity.render_mode()
            {
                d.draw_texture_pro(
                    tilesets.get(&tileset).unwrap(),
                    rrect(tile.x * 16.0, tile.y * 16.0, size.x * 16.0, size.y * 16.0),
                    rrect(
                        entity.top_left().x,
                        entity.top_left().y,
                        entity.size.x,
                        entity.size.y,
                    ),
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                );
            } else {
                d.draw_rectangle_v(entity.top_left(), entity.size, Color::BLUE);
            }
        }
    }

    Ok(())
}
