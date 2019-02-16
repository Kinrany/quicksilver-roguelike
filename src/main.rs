use quicksilver::{
  geom::{Rectangle, Shape, Vector},
  graphics::{Background::{Blended, Col, Img}, Color, Font, FontStyle, Image},
  lifecycle::{run, Asset, Settings, State, Window},
  Future, Result,
};

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
struct Tile {
  pos: Vector,
  glyph: char,
  color: Color,
}

fn generate_map(size: Vector) -> Vec<Tile> {
  let width = size.x as usize;
  let height = size.y as usize;
  let mut map = Vec::with_capacity(width * height);
  for x in 0..width {
    for y in 0..height {
      let mut tile = Tile {
        pos: Vector::new(x as f32, y as f32),
        glyph: '.',
        color: Color::BLACK,
      };

      if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
        tile.glyph = '#';
      };
      map.push(tile);
    }
  }
  map
}

#[derive(Clone, Debug, PartialEq)]
struct Entity {
  pos: Vector,
  glyph: char,
  color: Color,
  hp: i32,
  max_hp: i32,
}

fn generate_entities() -> Vec<Entity> {
  vec![
    Entity {
      pos: Vector::new(9, 6),
      glyph: 'g',
      color: Color::RED,
      hp: 1,
      max_hp: 1,
    },
    Entity {
      pos: Vector::new(2, 4),
      glyph: 'g',
      color: Color::RED,
      hp: 1,
      max_hp: 1,
    },
    Entity {
      pos: Vector::new(7, 5),
      glyph: '%',
      color: Color::PURPLE,
      hp: 0,
      max_hp: 0,
    },
    Entity {
      pos: Vector::new(4, 8),
      glyph: '%',
      color: Color::PURPLE,
      hp: 0,
      max_hp: 0,
    },
  ]
}

struct Game {
  title: Asset<Image>,
  mononoki_font_info: Asset<Image>,
  square_font_info: Asset<Image>,
  map_size: Vector,
  map: Vec<Tile>,
  entities: Vec<Entity>,
  player_entity_id: usize,
  tileset: Asset<HashMap<char, Image>>,
  tile_size_px: Vector,
}

impl State for Game {
  /// Load the assets and initialise the game
  fn new() -> Result<Self> {
    let font_mononoki = "mononoki-Regular.ttf";

    // create a prerendered image with title text
    let title = Asset::new(Font::load(font_mononoki).and_then(|font| {
      font.render("Quicksilver Roguelike", &FontStyle::new(72.0, Color::BLACK))
    }));

    // create a prerendered image with text about the Mononoki font
    let mononoki_font_info = Asset::new(Font::load(font_mononoki).and_then(|font| {
      font.render(
        "Mononoki font by Matthias Tellen, terms: SIL Open Font License 1.1",
        &FontStyle::new(20.0, Color::BLACK),
      )
    }));

    // create a prerendered image with text about the Square font
    let square_font_info = Asset::new(Font::load(font_mononoki).and_then(move |font| {
      font.render(
        "Square font by Wouter Van Oortmerssen, terms: CC BY 3.0",
        &FontStyle::new(20.0, Color::BLACK),
      )
    }));

    // map and entities
    let map_size = Vector::new(15, 15);
    let map = generate_map(map_size);
    let mut entities = generate_entities();

    // add entity "player"
    let player_entity_id = entities.len();
    entities.push(Entity {
      pos: Vector::new(5, 3),
      glyph: '@',
      color: Color::BLUE,
      hp: 3,
      max_hp: 5,
    });

    // create a prerendered tileset
    let font_square = "square.ttf";
    let game_glyphs = "#@g.%";
    let tile_size_px = Vector::new(24, 24);
    let tileset = Asset::new(Font::load(font_square).and_then(move |text| {
      let tiles = text
        .render(game_glyphs, &FontStyle::new(tile_size_px.y, Color::WHITE))
        .expect("Could not render the font tileset.");
      let mut tileset = HashMap::new();
      for (index, glyph) in game_glyphs.chars().enumerate() {
        let pos = (index as i32 * tile_size_px.x as i32, 0);
        let tile = tiles.subimage(Rectangle::new(pos, tile_size_px));
        tileset.insert(glyph, tile);
      }
      Ok(tileset)
    }));

    Ok(Self {
      title,
      mononoki_font_info,
      square_font_info,
      map_size,
      map,
      entities,
      player_entity_id,
      tileset,
      tile_size_px,
    })
  }

  /// Process keyboard and mouse, update the game state
  fn update(&mut self, _window: &mut Window) -> Result<()> {
    Ok(())
  }

  /// Draw stuff on the screen
  fn draw(&mut self, window: &mut Window) -> Result<()> {
    window.clear(Color::WHITE)?;

    // draw the image with title text
    self.title.execute(|image| {
      window.draw(
        &image
          .area()
          .with_center((window.screen_size().x as i32 / 2, 40)),
        Img(&image),
      );
      Ok(())
    })?;

    // draw the image with text about the Mononoki font
    self.mononoki_font_info.execute(|image| {
      window.draw(
        &image
          .area()
          .translate((2, window.screen_size().y as i32 - 60)),
        Img(&image),
      );
      Ok(())
    })?;

    // draw the image with text about the Square font
    self.square_font_info.execute(|image| {
      window.draw(
        &image
          .area()
          .translate((2, window.screen_size().y as i32 - 30)),
        Img(&image),
      );
      Ok(())
    })?;

    // create an alias
    let tile_size_px = self.tile_size_px;

    // coordinates of the upper left corner of the map on the screen
    let map_offset_px = Vector::new(50, 120);

    // draw the map
    let (tileset, map) = (&mut self.tileset, &self.map);
    tileset.execute(|tileset| {
      for tile in map.iter() {
        if let Some(image) = tileset.get(&tile.glyph) {
          let pos_px = tile.pos.times(tile_size_px);
          window.draw(
            &Rectangle::new(map_offset_px + pos_px, image.area().size()),
            Blended(&image, tile.color),
          );
        }
      }
      Ok(())
    })?;

    // draw the entities
    let (tileset, entities) = (&mut self.tileset, &self.entities);
    tileset.execute(|tileset| {
      for entity in entities.iter() {
        if let Some(image) = tileset.get(&entity.glyph) {
          let pos_px = entity.pos.times(tile_size_px);
          window.draw(
            &Rectangle::new(map_offset_px + pos_px, image.area().size()),
            Blended(&image, entity.color),
          );
        }
      }
      Ok(())
    })?;

    // draw the health bar
    {
      // calculate stuff
      let full_health_width_px = 100.0;
      let player = &self.entities[self.player_entity_id];
      let current_health_width_px =
        (player.hp as f32 / player.max_hp as f32) * full_health_width_px;
      let map_size_px = self.map_size.times(tile_size_px);
      let health_bar_pos_px = map_offset_px + Vector::new(map_size_px.x, 0.0);

      // draw full health bar
      window.draw(
        &Rectangle::new(health_bar_pos_px, (full_health_width_px, tile_size_px.y)),
        Col(Color::RED.with_alpha(0.5)),
      );

      // draw current health
      window.draw(
        &Rectangle::new(health_bar_pos_px, (current_health_width_px, tile_size_px.y)),
        Col(Color::RED),
      );
    }

    Ok(())
  }
}

fn main() {
  std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");

  let settings = Settings {
    scale: quicksilver::graphics::ImageScaleStrategy::Blur,
    ..Default::default()
  };
  run::<Game>("Quicksilver Roguelike", Vector::new(800, 600), settings);
}
