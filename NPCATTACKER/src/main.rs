use rustc_serialize::json::{self, Json};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uMod::plugin::{self, Plugin, TimerHandler};
use uMod::ujson::Value;
use uMod::umod::RustNative;

const RAID_INTERVAL_SECS: u64 = 14400; // 4 saatlik raid aralığı
const NPC_NAMES: [&str; 5] = ["Raider1", "Raider2", "Raider3", "Raider4", "Raider5"]; // NPC isimleri

#[derive(Debug)]
struct Base {
    location: (f64, f64, f64), // konum
    resources: HashMap<String, i32>, // kaynaklar
}

impl Base {
    // bir ev oluşturmak için kullanılacak olan yapılandırıcı fonksiyon
    fn new(location: (f64, f64, f64)) -> Self {
        Base {
            location,
            resources: HashMap::new(),
        }
    }
}

impl Default for Base {
    // varsayılan ev oluşturucu fonksiyonu
    fn default() -> Self {
        Base {
            location: (0.0, 0.0, 0.0),
            resources: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct Bases {
    bases: HashMap<u64, Base>, // evlerin saklandığı hashmap
}

impl Bases {
    fn new() -> Self {
        Bases {
            bases: HashMap::new(),
        }
    }

    fn add_base(&mut self, base_id: u64, location: (f64, f64, f64)) {
        self.bases.insert(base_id, Base::new(location));
    }

    fn remove_base(&mut self, base_id: u64) {
        self.bases.remove(&base_id);
    }

    fn get_base_location(&self, base_id: u64) -> Option<&(f64, f64, f64)> {
        self.bases.get(&base_id).map(|base| &base.location)
    }

    fn add_resource(&mut self, base_id: u64, resource_name: &str, amount: i32) -> bool {
        if let Some(base) = self.bases.get_mut(&base_id) {
            *base.resources.entry(resource_name.to_owned()).or_insert(0) += amount;
            true
        } else {
            false
        }
    }

    fn get_resources(&self, base_id: u64) -> Option<&HashMap<String, i32>> {
        self.bases.get(&base_id).map(|base| &base.resources)
    }
}

#[derive(Debug)]
struct NpcRaiders {
    bases: Arc<Bases>, // tüm evleri tutacak olan bases yapısı
}

impl NpcRaiders {
    fn new(bases: Arc<Bases>) -> Self {
        NpcRaiders { bases }
    }

    fn raid_base(&self, base_id: u64) {
        if let Some(location) = self.bases.get_base_location(base_id) {
            let mut rng = rand::thread_rng();
            let num_raiders = rng.gen_range(1..6); // 1-5 arası NPC'ler gönderilir
            let mut resources_stolen = HashMap::new(); // çalınacak kaynaklar

            // NPC'lerin evi yağmalaması
            for _ in 0..num_raiders {
                let raider = NPC_NAMES[rng.gen_range(0..NPC_NAMES.len())];
                println!("{} evi yağmalamaya gidiyor...", raider);
                thread::sleep(Duration::from_secs(3)); // NPC'lerin eve varış süresi

                // rastgele bir kaynak seç ve çal
                if let Some(resources) = self.bases.get_resources(base_id) {
                    if let Some((resource_name, resource_amount)) = resources
                        .iter()
                        .skip(rng.gen_range(0..resources.len()))
                        .next()
                    {
                        println!(
                            "{} {} adet {} çaldı!",
                            raider, resource_amount, resource_name
                        );
                        *resources_stolen.entry(resource_name.to_owned()).or_insert(0) +=
                            resource_amount;
                    }
                }
            }

            // çalınan kaynakları evden kaldır
            for (resource_name, resource_amount) in resources_stolen.iter() {
                println!(
                    "{} adet {} çalındı. Evden kaldırılıyor...",
                    resource_amount, resource_name
                );
                self.bases
                    .add_resource(base_id, resource_name, -resource_amount);
            }

            // yağmalama sonrası işlemler
            println!("Yağmalama sona erdi.");
        }
    }
}

// Rust uMod API'leri için gereken trait'leri uygulayalım
impl Plugin for NpcRaiders {
    fn new(args: &str) -> Result<Self, Box<dyn Error>> {
        let bases = Arc::new(Bases::new());
        let npc_raiders = NpcRaiders::new(bases.clone());

        // 5 saniye sonra ilk yağma gerçekleşecek şekilde bir zamanlayıcı başlat
        plugin::Timer::new(
            Duration::from_secs(5),
            Some(Duration::from_secs(RAID_INTERVAL_SECS)),
            move || {
                let base_ids: Vec<u64> = bases.bases.keys().cloned().collect();
                let mut rng = rand::thread_rng();

                // rastgele bir evi seç ve yağmalama başlat
                if let Some(base_id) = base_ids.choose(&mut rng) {
                    npc_raiders.raid_base(*base_id);
                }
            },
        )
        .start();

        Ok(npc_raiders)
    }
}
