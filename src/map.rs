use rltk::{Algorithm2D, BaseMap, Point, Rect};
use std::convert::TryInto;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub known_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Manhattan.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map {
    pub fn get_index(&self, x: i32, y: i32) -> usize {
        ((y * self.width) + x) as usize
    }

    pub fn set_blocked_tiles(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            let is_blocked = *tile == TileType::Wall;
            self.blocked_tiles[index] = is_blocked;
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }

        !self.blocked_tiles[self.get_index(x, y)]
    }

    fn build_room(&mut self, room: Rect) {
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let index = self.get_index(x, y);
                self.tiles[index] = TileType::Floor;
            }
        }

        self.rooms.push(room);
    }

    /// Create a hallway of TileType::Floor between the given start and end points
    /// The hallway will always be built horizontally from the start position and vertically from the end position
    fn build_hallway(&mut self, start: Point, end: Point) {
        let xrange;
        let yrange;

        if start.x > end.x {
            xrange = (end.x - start.x)..=0;
        } else {
            xrange = 0..=(end.x - start.x);
        }

        if start.y > end.y {
            yrange = 0..=(start.y - end.y);
        } else {
            yrange = (start.y - end.y)..=0;
        }

        for dx in xrange {
            let next_x = start.x + dx;
            let next_y = start.y;

            let index = self.get_index(next_x, next_y);
            self.tiles[index] = TileType::Floor;
        }

        for dy in yrange {
            let next_x = end.x;
            let next_y = end.y + dy;

            let index = self.get_index(next_x, next_y);
            self.tiles[index] = TileType::Floor;
        }
    }
}

pub fn build_rogue_map(width: i32, height: i32) -> Map {
    let dim = (width * height).try_into().unwrap();
    let mut map = Map {
        tiles: vec![TileType::Wall; dim],
        rooms: vec![],
        width: width,
        height: height,
        known_tiles: vec![false; dim],
        visible_tiles: vec![false; dim],
        blocked_tiles: vec![false; dim],
    };

    let mut rng = rltk::RandomNumberGenerator::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_ROOM_WIDTH: i32 = 3;
    const MAX_ROOM_WIDTH: i32 = 12;
    const MIN_ROOM_HEIGHT: i32 = 3;
    const MAX_ROOM_HEIGHT: i32 = 12;

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_ROOM_WIDTH, MAX_ROOM_WIDTH);
        let h = rng.range(MIN_ROOM_HEIGHT, MAX_ROOM_HEIGHT);
        let x = rng.range(1, map.width - w - 1);
        let y = rng.range(1, map.height - h - 1);

        let new_room = Rect::with_size(x, y, w, h);
        let mut quit = false;

        for other_rooms in map.rooms.iter() {
            if other_rooms.intersect(&new_room) {
                quit = true;
            }
        }

        if quit {
            continue;
        }

        map.build_room(new_room);
        if map.rooms.len() > 1 {
            let new_center = new_room.center();
            let prev_center = map.rooms[map.rooms.len() - 2].center();

            if rng.rand::<f32>() > 0.5 {
                map.build_hallway(prev_center, new_center);
            } else {
                map.build_hallway(new_center, prev_center);
            }
        }
    }

    map.set_blocked_tiles();
    map
}
