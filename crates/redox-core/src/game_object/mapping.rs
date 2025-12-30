use super::types::{GameObjectType, HitboxShape};

pub fn get_hitbox_for_id(id: i32) -> (HitboxShape, f32, f32) {
	match id {
		// Blocks
		1..=4
		| 6..=7
		| 63
		| 69..=72
		| 74..=78
		| 81..=83
		| 90..=96
		| 116..=119
		| 121..=122
		| 146
		| 160..=163
		| 165..=169
		| 173
		| 175
		| 207..=210
		| 212..=213
		| 247..=250
		| 252..=258
		| 260..=261
		| 263..=265
		| 267..=272
		| 274..=275
		| 467
		| 469..=471
		| 1203..=1204
		| 1209..=1210
		| 1221..=1222
		| 1226 => (HitboxShape::Rectangle, 30.0, 30.0),
		64 | 195 | 206 | 220 | 661 | 1155..=1157 | 1208 | 1910 => {
			(HitboxShape::Rectangle, 15.0, 15.0)
		}
		40 | 147 | 215 | 369..=370 | 1903..=1905 => (HitboxShape::Rectangle, 30.0, 14.0),
		170..=172 | 174 | 192 => (HitboxShape::Rectangle, 30.0, 21.0),
		468 | 475 | 1260 => (HitboxShape::Rectangle, 30.0, 1.5),
		62 | 65 | 66 | 68 => (HitboxShape::Rectangle, 30.0, 16.0),
		1202 | 1262 => (HitboxShape::Rectangle, 30.0, 3.0),
		1220 | 1264 => (HitboxShape::Rectangle, 30.0, 6.0),
		196 | 219 | 1911 => (HitboxShape::Rectangle, 15.0, 8.0),
		204 => (HitboxShape::Rectangle, 8.0, 15.0),
		662..=664 => (HitboxShape::Rectangle, 30.0, 15.0),
		1561 => (HitboxShape::Rectangle, 30.0, 10.0),
		1567 => (HitboxShape::Rectangle, 15.0, 10.0),
		1566 => (HitboxShape::Rectangle, 12.0, 12.0),
		1565 => (HitboxShape::Rectangle, 17.0, 17.0),
		1227 => (HitboxShape::Rectangle, 30.0, 7.0),
		328 => (HitboxShape::Rectangle, 22.0, 22.0),
		197 => (HitboxShape::Rectangle, 22.0, 21.0),
		194 => (HitboxShape::Rectangle, 21.0, 21.0),
		176 => (HitboxShape::Rectangle, 14.0, 21.0),
		1562 => (HitboxShape::Rectangle, 30.0, 2.0),
		1343 => (HitboxShape::Rectangle, 25.0, 3.0),
		1340 => (HitboxShape::Rectangle, 27.0, 2.0),
		34 => (HitboxShape::Rectangle, 37.0, 23.0),
		143 => (HitboxShape::Rectangle, 30.0, 30.0),

		// Spikes/Hazards
		8 | 144 | 177 | 216 => (HitboxShape::Rectangle, 6.0, 12.0),
		103 | 145 | 218 => (HitboxShape::Rectangle, 4.0, 7.6),
		39 | 205 | 217 => (HitboxShape::Rectangle, 6.0, 5.6),
		720 | 991 | 1731 | 1733 => (HitboxShape::Rectangle, 2.4, 3.2),
		61 | 446 | 1719 | 1728 => (HitboxShape::Rectangle, 9.0, 7.2),
		365 | 667 | 1716 | 1730 => (HitboxShape::Rectangle, 9.0, 6.0),
		392 | 458..=459 => (HitboxShape::Rectangle, 2.6, 4.8),
		768 | 1727 => (HitboxShape::Rectangle, 4.5, 5.2),
		447 | 1729 => (HitboxShape::Rectangle, 5.2, 7.2),
		135 | 1711 => (HitboxShape::Rectangle, 14.1, 20.0),
		422 | 1726 => (HitboxShape::Rectangle, 6.0, 4.4),
		244 | 1721 => (HitboxShape::Rectangle, 6.0, 6.8),
		243 | 1720 => (HitboxShape::Rectangle, 6.0, 7.2),
		421 | 1725 => (HitboxShape::Rectangle, 9.0, 5.2),
		9 | 1715 => (HitboxShape::Rectangle, 9.0, 10.8),
		989 | 1732 => (HitboxShape::Rectangle, 9.0, 12.0),
		1714 => (HitboxShape::Rectangle, 11.4, 16.4),
		1712 => (HitboxShape::Rectangle, 13.5, 22.4),
		368 | 1722 => (HitboxShape::Rectangle, 9.0, 4.0),
		1713 => (HitboxShape::Rectangle, 11.7, 20.0),
		178 => (HitboxShape::Rectangle, 6.0, 6.4),
		919 => (HitboxShape::Rectangle, 25.0, 6.0),
		179 => (HitboxShape::Rectangle, 4.0, 8.0),

		// Sawblades
		88 | 186 | 740 | 1705 => (HitboxShape::Circle, 32.3, 32.3),
		89 | 1706 => (HitboxShape::Circle, 21.6, 21.6),
		98 | 1707 => (HitboxShape::Circle, 12.0, 12.0),
		183 => (HitboxShape::Circle, 15.66, 15.66),
		184 => (HitboxShape::Circle, 20.4, 20.4),
		185 => (HitboxShape::Circle, 2.85, 2.85),
		187 | 741 => (HitboxShape::Circle, 21.96, 21.96),
		188 | 742 => (HitboxShape::Circle, 12.6, 12.6),
		397 | 1708 => (HitboxShape::Circle, 28.9, 28.9),
		398 | 1709 => (HitboxShape::Circle, 17.44, 17.44),
		399 | 1710 => (HitboxShape::Circle, 12.9, 12.9),
		675 | 1734 => (HitboxShape::Circle, 32.0, 32.0),
		676 | 1735 => (HitboxShape::Circle, 17.51, 17.51),
		677 | 1736 => (HitboxShape::Circle, 12.48, 12.48),
		678 => (HitboxShape::Circle, 30.4, 30.4),
		679 => (HitboxShape::Circle, 18.54, 18.54),
		680 => (HitboxShape::Circle, 10.8, 10.8),
		918 => (HitboxShape::Circle, 24.0, 24.0),
		1582..=1583 => (HitboxShape::Circle, 4.0, 4.0),
		1619 => (HitboxShape::Circle, 25.0, 25.0),
		1620 => (HitboxShape::Circle, 15.0, 15.0),
		1701..=1703 => (HitboxShape::Circle, 6.0, 6.0),

		// Pads
		35 => (HitboxShape::Rectangle, 25.0, 4.0),
		140 => (HitboxShape::Rectangle, 25.0, 5.0),
		67 => (HitboxShape::Rectangle, 25.0, 6.0),

		// Orbs
		36 | 84 | 141 => (HitboxShape::Rectangle, 36.0, 36.0),

		// Portals
		12 | 13 | 47 | 111 | 660 => (HitboxShape::Rectangle, 34.0, 86.0),
		10 | 11 => (HitboxShape::Rectangle, 25.0, 75.0),
		99 | 101 => (HitboxShape::Rectangle, 31.0, 90.0),
		200 => (HitboxShape::Rectangle, 35.0, 44.0),
		201 => (HitboxShape::Rectangle, 33.0, 56.0),
		202 => (HitboxShape::Rectangle, 51.0, 56.0),
		203 => (HitboxShape::Rectangle, 65.0, 56.0),
		1334 => (HitboxShape::Rectangle, 69.0, 56.0),

		// Slopes
		289 | 294 | 299 | 305 | 309 | 315 | 321 | 326 | 331 | 337 | 343 | 349 | 353 | 371 | 483
		| 492 | 651 | 665 | 673 | 709 | 711 | 726 | 728 | 886 | 1338 | 1341 | 1344 | 1723
		| 1743 | 1745 | 1747 | 1749 | 1906 => (HitboxShape::Rectangle, 30.0, 30.0),
		363 | 1717 => (HitboxShape::Rectangle, 30.0, 30.0),
		291 | 295 | 301 | 307 | 311 | 317 | 323 | 327 | 333 | 339 | 345 | 351 | 355 | 367 | 372
		| 484 | 493 | 652 | 666 | 674 | 710 | 712 | 727 | 729 | 887 | 1339 | 1342 | 1345 | 1724
		| 1744 | 1746 | 1748 | 1750 | 1907 => (HitboxShape::Rectangle, 60.0, 30.0),
		364 | 366 | 1718 => (HitboxShape::Rectangle, 60.0, 30.0),

		_ => (HitboxShape::Rectangle, 30.0, 30.0),
	}
}

pub fn get_object_type_for_id(id: i32) -> GameObjectType {
	match id {
		// Blocks
		1..=4
		| 6..=7
		| 63
		| 69..=72
		| 74..=78
		| 81..=83
		| 90..=96
		| 116..=119
		| 121..=122
		| 146
		| 160..=163
		| 165..=169
		| 173
		| 175
		| 207..=210
		| 212..=213
		| 247..=250
		| 252..=258
		| 260..=261
		| 263..=265
		| 267..=272
		| 274..=275
		| 467
		| 469..=471
		| 1203..=1204
		| 1209..=1210
		| 1221..=1222
		| 1226 => GameObjectType::Solid,
		64 | 195 | 206 | 220 | 661 | 1155..=1157 | 1208 | 1910 => GameObjectType::Solid,
		40 | 147 | 215 | 369..=370 | 1903..=1905 => GameObjectType::Solid,
		170..=172 | 174 | 192 => GameObjectType::Solid,
		468 | 475 | 1260 => GameObjectType::Solid,
		62 | 65 | 66 | 68 => GameObjectType::Solid,
		1202 | 1262 => GameObjectType::Solid,
		1220 | 1264 => GameObjectType::Solid,
		196 | 219 | 1911 => GameObjectType::Solid,
		204 => GameObjectType::Solid,
		662..=664 => GameObjectType::Solid,
		1561 | 1567 | 1566 | 1565 | 1227 | 328 | 197 | 194 | 176 | 1562 | 1343 | 1340 | 34 => {
			GameObjectType::Solid
		}
		143 => GameObjectType::Breakable,

		// Spikes/Hazards
		8
		| 144
		| 177
		| 216
		| 103
		| 145
		| 218
		| 39
		| 205
		| 217
		| 720
		| 991
		| 1731
		| 1733
		| 61
		| 446
		| 1719
		| 1728
		| 365
		| 667
		| 1716
		| 1730
		| 392
		| 458..=459
		| 768
		| 1727
		| 447
		| 1729
		| 135
		| 1711
		| 422
		| 1726
		| 244
		| 1721
		| 243
		| 1720
		| 421
		| 1725
		| 9
		| 1715
		| 989
		| 1732
		| 1714
		| 1712
		| 368
		| 1722
		| 1713
		| 178
		| 919
		| 179 => GameObjectType::Hazard,
		363 | 1717 | 364 | 366 | 1718 => GameObjectType::Hazard,

		// Sawblades
		88
		| 186
		| 740
		| 1705
		| 89
		| 1706
		| 98
		| 1707
		| 183
		| 184
		| 185
		| 187
		| 741
		| 188
		| 742
		| 397
		| 1708
		| 398
		| 1709
		| 399
		| 1710
		| 675
		| 1734
		| 676
		| 1735
		| 677
		| 1736
		| 678
		| 679
		| 680
		| 918
		| 1582..=1583
		| 1619
		| 1620
		| 1701..=1703 => GameObjectType::Sawblade,

		// Portals
		11 => GameObjectType::InverseGravityPortal,
		10 => GameObjectType::NormalGravityPortal,
		13 => GameObjectType::ShipPortal,
		12 => GameObjectType::CubePortal,
		47 => GameObjectType::BallPortal,
		111 => GameObjectType::UfoPortal,
		660 => GameObjectType::WavePortal,
		99 => GameObjectType::MiniSizePortal,
		101 => GameObjectType::RegularSizePortal,
		200..=203 | 1334 => GameObjectType::Special,

		// Slopes
		289 | 294 | 299 | 305 | 309 | 315 | 321 | 326 | 331 | 337 | 343 | 349 | 353 | 371 | 483
		| 492 | 651 | 665 | 673 | 709 | 711 | 726 | 728 | 886 | 1338 | 1341 | 1344 | 1723
		| 1743 | 1745 | 1747 | 1749 | 1906 | 291 | 295 | 301 | 307 | 311 | 317 | 323 | 327
		| 333 | 339 | 345 | 351 | 355 | 367 | 372 | 484 | 493 | 652 | 666 | 674 | 710 | 712
		| 727 | 729 | 887 | 1339 | 1342 | 1345 | 1724 | 1744 | 1746 | 1748 | 1750 | 1907 => {
			GameObjectType::Slope
		}

		// Pads
		35 => GameObjectType::YellowJumpPad,
		140 => GameObjectType::PinkJumpPad,
		67 => GameObjectType::GravityPad,

		// Orbs
		36 => GameObjectType::YellowJumpRing,
		84 => GameObjectType::PinkJumpRing,
		141 => GameObjectType::GravityRing,

		_ => GameObjectType::Unknown,
	}
}
