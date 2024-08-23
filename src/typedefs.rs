use crate::definitions::*;

fn preprocess_header(header: &str, preferences: &Preferences) -> String {
    header.replace(
        "[SERDE]",
        if preferences.serde {
            "serde::Serialize, serde::Deserialize, "
        } else {
            ""
        },
    )
}

pub fn generate_defs(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    project: &LdtkJson,
    code: &mut Scope,
) -> Result<()> {
    let header = preprocess_header(include_str!("templates/header.rs"), preferences);
    let header = header.replace(
        "define_vectors!();",
        &if let Some(vector_type) = &preferences.vector {
            format!(
                            r"
type UVec2 = {};
type IVec2 = {};
type FVec2 = {};
",
            vector_type.replace("<T>", "<u32>"),
            vector_type.replace("<T>", "<i32>"),
            vector_type.replace("<T>", "<f32>"),
                        )
        } else {
            preprocess_header(include_str!("templates/math.rs"), preferences)
        },
    );
    let header = header.replace(
        "define_colors!();",
        &if let Some(color_type) = &preferences.color {
            format!("type Color = {color_type};")
        } else {
            preprocess_header(include_str!("templates/color.rs"), preferences)
        },
    );
    macro_rules! replace_get_layer {
        ($header: ident, $mutability: literal) => {
            $header.replace(
                concat!(
                    "            LAYER_INDEX => GET_LAYER",
                    $mutability,
                    "!(),\n"
                ),
                &project
                    .defs
                    .layers
                    .iter()
                    .filter(|layer| matches!(layer.purple_type, Type::Entities))
                    .enumerate()
                    .map(|(index, layer)| {
                        format!(
                            concat!(
                                "            {} => level.{}.get",
                                $mutability,
                                "(self.entity),\n"
                            ),
                            index,
                            preferences.to_case(&layer.identifier, Case::Snake)
                        )
                    })
                    .collect::<String>(),
            )
        };
    }
    let header = replace_get_layer!(header, "");
    let header = replace_get_layer!(header, "_mut");
    code.raw(header);

    for tileset in &project.defs.tilesets {
        definitions.tilesets.insert(
            tileset.uid,
            RsTilesetDefinition {
                tile_size: tileset.tile_grid_size as _,
            },
        );
    }

    code.raw("/* --- Definitions --- */");
    code.raw("/* Enums */");
    for enum_json in &project.defs.enums {
        let enum_rs = code.new_enum(&enum_json.identifier).vis("pub");
        derive_rust_object!(enum_rs preferences.serde, Hash !partial Eq, Ord);
        for value in &enum_json.values {
            enum_rs.new_variant(&value.id);
        }

        macro_rules! generate_get_const {
            ($impl: ident $fn: ident -> $ret: ty; $variant: ident => $line: expr) => {
                $impl
                    .new_fn(stringify!($fn))
                    .vis("pub")
                    .arg_ref_self()
                    .ret(stringify!($ret))
                    .push_block({
                        let mut match_block = Block::new("match self");
                        for $variant in &enum_json.values {
                            match_block.line(format!("Self::{} => {},", $variant.id, $line));
                        }
                        match_block
                    });
            };
        }

        let enum_impl = code.new_impl(&enum_json.identifier);
        generate_get_const!(enum_impl color -> Option<Color>; variant => if variant.color >= 0 {
            format!("Some(<Color as ColorImpl>::from_hex({}))", variant.color)
        } else {
            "None".to_owned()
        });

        if let Some(tileset) = enum_json.icon_tileset_uid {
            enum_impl.associate_const("TILESET_ID", "TilesetID", tileset.to_string(), "pub");
            generate_get_const!(enum_impl icon -> Option<UVec2>; variant => if let Some(tile) = &variant.tile_rect {
                let tileset = definitions.tilesets.get(&tileset).context("Enum icon tileset was not found!")?;
                format!("Some(<UVec2 as VectorImpl>::new({} as _, {} as _))", tile.x as u32 / tileset.tile_size, tile.y as u32 / tileset.tile_size)
            } else {
                "None".to_owned()
            });
        }
    }
    Ok(())
}
