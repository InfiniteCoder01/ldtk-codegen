pub mod demo;
use demo::layer::*;
use demo::math::*;
use demo::*;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init().size(1632, 928).title("Hello, World").build();
    let world = World::load();
    let level = &world.entities_demo;
    let mut tilesets = std::collections::HashMap::new();
    for tileset in [&world.icons, &world.inca_front, &world.inca_back] {
        tilesets.insert(
            tileset.id(),
            rl.load_texture(
                &thread,
                std::path::Path::new("src")
                    .join(tileset.path())
                    .to_str()
                    .unwrap(),
            )
            .unwrap(),
        );
    }

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(27, 27, 46, 255));
        let mut d = d.begin_mode2D(Camera2D {
            offset: Vector2::zero(),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 2.0,
        });

        for y in 0..level.collisions.size().y {
            for x in 0..level.collisions.size().x {
                let tile_pos = Vec2::new(x, y);
                for tile in level.collisions.get_tileset_tile(tile_pos) {
                    d.draw_texture_rec(
                        tilesets.get(&level.collisions.tileset_id()).unwrap(),
                        rrect(
                            tile.position().x * 16,
                            tile.position().y * 16,
                            16 * if tile.flip().horizontal() { -1 } else { 1 },
                            16 * if tile.flip().vertical() { -1 } else { 1 },
                        ),
                        (tile_pos * Collisions::GRID_SIZE).casted::<Vector2>(),
                        Color::WHITE,
                    );
                }
            }
        }
        for entity in level.triggerables.entities() {
            d.draw_rectangle_v(
                entity.top_left().casted::<Vector2>(),
                entity.size.casted::<Vector2>(),
                Color::new(0, 255, 0, 30),
            );
        }
        for entity in level.game_entities.entities() {
            if let Entity::Enemy(enemy) = &entity.entity {
                let tile = enemy.enemy_type.icon().unwrap();
                d.draw_texture_pro(
                    tilesets.get(&MonsterType::TILESET_ID).unwrap(),
                    rrect(tile.x * 16, tile.y * 16, 16, 16),
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
            } else if let RenderMode::Tile {
                tileset,
                tile,
                size,
            } = entity.entity.render_mode()
            {
                d.draw_texture_pro(
                    tilesets.get(&tileset).unwrap(),
                    rrect(tile.x * 16, tile.y * 16, size.x * 16, size.y * 16),
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
                d.draw_rectangle_v(
                    entity.top_left().casted::<Vector2>(),
                    entity.size.casted::<Vector2>(),
                    Color::BLUE,
                );
            }
        }
    }
}
