const D = {
            n: "Human Languages",
            d: -100000,
            s: 0,
            x: "extinct",
            b: "root",
            i: "~7,000 living languages today",
            c: [
                // INDO-EUROPEANW
                {
                    n: "Indo-European", d: -4500, b: "ie", i: "3.2B speakers, largest family",
                    c: [
                        {
                            n: "Anatolian", d: -4200, e: 400, x: "extinct", b: "ie", i: "First split, all extinct", c: [
                                { n: "Hittite", d: -1650, e: -1178, x: "extinct", b: "ie", i: "Oldest attested IE" },
                                { n: "Luwian", d: -1400, e: -600, x: "extinct", b: "ie" },
                                { n: "Lycian", d: -500, e: -200, x: "extinct", b: "ie" }
                            ]
                        },
                        {
                            n: "Tocharian", d: -3500, e: 1000, x: "extinct", b: "ie", i: "Tarim Basin, China", c: [
                                { n: "Tocharian A", d: 500, e: 900, x: "extinct", b: "ie" },
                                { n: "Tocharian B", d: 500, e: 1000, x: "extinct", b: "ie" }
                            ]
                        },
                        {
                            n: "Celtic", d: -2800, b: "ie", i: "Once across Europe", c: [
                                { n: "Gaulish", d: -500, e: 500, x: "extinct", b: "ie" },
                                { n: "Irish", d: 400, s: 1770, x: "living", b: "ie" },
                                { n: "Scottish Gaelic", d: 500, s: 57, x: "living", b: "ie" },
                                { n: "Welsh", d: 700, s: 880, x: "living", b: "ie" },
                                { n: "Breton", d: 500, s: 210, x: "living", b: "ie" }
                            ]
                        },
                        {
                            n: "Italic", d: -2500, b: "ie", i: "Latin → Romance", c: [
                                {
                                    n: "Latin", d: -700, b: "ie", i: "Evolved into Romance", c: [
                                        { n: "Spanish", d: 900, s: 486000, x: "living", b: "ie", i: "486M native" },
                                        { n: "Portuguese", d: 900, s: 264000, x: "living", b: "ie", i: "264M speakers" },
                                        { n: "French", d: 842, s: 77000, x: "living", b: "ie", i: "77M native" },
                                        { n: "Italian", d: 960, s: 64000, x: "living", b: "ie" },
                                        { n: "Romanian", d: 800, s: 24000, x: "living", b: "ie" },
                                        { n: "Catalan", d: 900, s: 4100, x: "living", b: "ie" }
                                    ]
                                }
                            ]
                        },
                        {
                            n: "Germanic", d: -2500, b: "ie", i: "550M speakers", c: [
                                { n: "English", d: 1500, s: 380000, x: "living", b: "ie", i: "380M native, 1.5B total" },
                                { n: "German", d: 1350, s: 100000, x: "living", b: "ie", i: "100M speakers" },
                                { n: "Dutch", d: 1150, s: 25000, x: "living", b: "ie" },
                                { n: "Swedish", d: 1100, s: 10200, x: "living", b: "ie" },
                                { n: "Norwegian", d: 1100, s: 5300, x: "living", b: "ie" },
                                { n: "Danish", d: 1100, s: 5500, x: "living", b: "ie" },
                                { n: "Afrikaans", d: 1700, s: 7200, x: "living", b: "ie" },
                                { n: "Yiddish", d: 1000, s: 1500, x: "living", b: "ie" },
                                { n: "Icelandic", d: 1100, s: 330, x: "living", b: "ie" }
                            ]
                        },
                        {
                            n: "Baltic", d: -2000, b: "ie", i: "Most archaic living IE", c: [
                                { n: "Lithuanian", d: 1000, s: 2800, x: "living", b: "ie" },
                                { n: "Latvian", d: 1000, s: 1500, x: "living", b: "ie" }
                            ]
                        },
                        {
                            n: "Slavic", d: -1500, b: "ie", i: "315M speakers", c: [
                                { n: "Russian", d: 1400, s: 154000, x: "living", b: "ie", i: "154M speakers" },
                                { n: "Ukrainian", d: 1400, s: 33000, x: "living", b: "ie" },
                                { n: "Polish", d: 1000, s: 45000, x: "living", b: "ie" },
                                { n: "Czech", d: 1000, s: 10700, x: "living", b: "ie" },
                                { n: "Serbian", d: 900, s: 9000, x: "living", b: "ie" },
                                { n: "Bulgarian", d: 900, s: 7000, x: "living", b: "ie" },
                                { n: "Croatian", d: 900, s: 5500, x: "living", b: "ie" }
                            ]
                        },
                        {
                            n: "Indo-Aryan", d: -2000, b: "ie", i: "800M+ in South Asia", c: [
                                { n: "Hindi", d: 1200, s: 345000, x: "living", b: "ie", i: "345M native" },
                                { n: "Bengali", d: 1000, s: 234000, x: "living", b: "ie", i: "234M speakers" },
                                { n: "Punjabi", d: 1000, s: 125000, x: "living", b: "ie" },
                                { n: "Marathi", d: 1000, s: 83000, x: "living", b: "ie" },
                                { n: "Urdu", d: 1200, s: 70000, x: "living", b: "ie" },
                                { n: "Gujarati", d: 1200, s: 56000, x: "living", b: "ie" },
                                { n: "Nepali", d: 1200, s: 16000, x: "living", b: "ie" },
                                { n: "Sinhalese", d: 300, s: 17000, x: "living", b: "ie" }
                            ]
                        },
                        {
                            n: "Iranian", d: -2000, b: "ie", i: "150M speakers", c: [
                                { n: "Persian", d: 800, s: 77000, x: "living", b: "ie", i: "77M speakers" },
                                { n: "Pashto", d: 1000, s: 50000, x: "living", b: "ie" },
                                { n: "Kurdish", d: 800, s: 30000, x: "living", b: "ie" }
                            ]
                        },
                        {
                            n: "Hellenic", d: -2200, b: "ie", c: [
                                { n: "Greek", d: 1453, s: 13500, x: "living", b: "ie", i: "13.5M speakers" }
                            ]
                        },
                        {
                            n: "Armenian", d: -1800, b: "ie", c: [
                                { n: "Armenian", d: 405, s: 4000, x: "living", b: "ie" }
                            ]
                        },
                        {
                            n: "Albanian", d: -1500, b: "ie", c: [
                                { n: "Albanian", d: 1000, s: 7500, x: "living", b: "ie" }
                            ]
                        }
                    ]
                },

                // SINO-TIBETAN
                {
                    n: "Sino-Tibetan", d: -6000, b: "sino", i: "1.4B speakers, 2nd largest",
                    c: [
                        {
                            n: "Sinitic", d: -3000, b: "sino", i: "Chinese languages", c: [
                                { n: "Mandarin", d: 1300, s: 920000, x: "living", b: "sino", i: "920M native, most spoken" },
                                { n: "Wu", d: 500, s: 82000, x: "living", b: "sino", i: "Shanghainese" },
                                { n: "Cantonese", d: 200, s: 85000, x: "living", b: "sino", i: "Hong Kong, Guangdong" },
                                { n: "Min", d: 500, s: 75000, x: "living", b: "sino", i: "Hokkien, Teochew" },
                                { n: "Jin", d: 500, s: 47000, x: "living", b: "sino" },
                                { n: "Hakka", d: 300, s: 44000, x: "living", b: "sino" },
                                { n: "Gan", d: 500, s: 22000, x: "living", b: "sino" },
                                { n: "Xiang", d: 500, s: 38000, x: "living", b: "sino" }
                            ]
                        },
                        {
                            n: "Tibeto-Burman", d: -3000, b: "sino", i: "~400 languages", c: [
                                { n: "Burmese", d: 800, s: 33000, x: "living", b: "sino", i: "Myanmar" },
                                { n: "Tibetan", d: 600, s: 6000, x: "living", b: "sino" },
                                { n: "Karen", d: 500, s: 4000, x: "living", b: "sino" },
                                { n: "Meitei", d: 500, s: 1800, x: "living", b: "sino" },
                                { n: "Dzongkha", d: 800, s: 640, x: "living", b: "sino", i: "Bhutan" }
                            ]
                        }
                    ]
                },

                // AFRO-ASIATIC
                {
                    n: "Afro-Asiatic", d: -12000, b: "afro", i: "500M speakers",
                    c: [
                        {
                            n: "Semitic", d: -4000, b: "afro", i: "Arabic, Hebrew, Amharic", c: [
                                { n: "Arabic", d: 300, s: 310000, x: "living", b: "afro", i: "310M native, liturgical" },
                                { n: "Amharic", d: 1000, s: 32000, x: "living", b: "afro", i: "Ethiopia" },
                                { n: "Hebrew", d: -1000, s: 9000, x: "living", b: "afro", i: "Revived 19th c." },
                                { n: "Tigrinya", d: 500, s: 9000, x: "living", b: "afro" },
                                { n: "Maltese", d: 1000, s: 520, x: "living", b: "afro" }
                            ]
                        },
                        {
                            n: "Berber", d: -3000, b: "afro", i: "North Africa", c: [
                                { n: "Tamazight", d: 500, s: 8000, x: "living", b: "afro" },
                                { n: "Kabyle", d: 500, s: 6000, x: "living", b: "afro" },
                                { n: "Tachelhit", d: 500, s: 5000, x: "living", b: "afro" }
                            ]
                        },
                        {
                            n: "Cushitic", d: -5000, b: "afro", i: "Horn of Africa", c: [
                                { n: "Oromo", d: 500, s: 37000, x: "living", b: "afro", i: "Ethiopia" },
                                { n: "Somali", d: 500, s: 22000, x: "living", b: "afro" }
                            ]
                        },
                        {
                            n: "Chadic", d: -4000, b: "afro", c: [
                                { n: "Hausa", d: 500, s: 77000, x: "living", b: "afro", i: "77M, Nigeria lingua franca" }
                            ]
                        },
                        {
                            n: "Egyptian", d: -3000, e: 1600, x: "extinct", b: "afro", i: "Ancient Egypt → Coptic", c: [
                                { n: "Coptic", d: 200, s: 0.01, x: "living", b: "afro", i: "Liturgical only" }
                            ]
                        }
                    ]
                },

                // NIGER-CONGO
                {
                    n: "Niger-Congo", d: -10000, b: "niger", i: "700M speakers, 1,500+ langs",
                    c: [
                        {
                            n: "Bantu", d: -3000, b: "niger", i: "500+ languages", c: [
                                { n: "Swahili", d: 1000, s: 16000, x: "living", b: "niger", i: "100M+ L2 speakers" },
                                { n: "Shona", d: 1000, s: 12000, x: "living", b: "niger" },
                                { n: "Zulu", d: 1500, s: 12000, x: "living", b: "niger" },
                                { n: "Xhosa", d: 1500, s: 8200, x: "living", b: "niger" },
                                { n: "Kinyarwanda", d: 1000, s: 12000, x: "living", b: "niger" },
                                { n: "Kikuyu", d: 1000, s: 7000, x: "living", b: "niger" },
                                { n: "Lingala", d: 1800, s: 15000, x: "living", b: "niger" }
                            ]
                        },
                        {
                            n: "Atlantic-Congo", d: -5000, b: "niger", c: [
                                { n: "Yoruba", d: 1000, s: 45000, x: "living", b: "niger", i: "Nigeria" },
                                { n: "Igbo", d: 1000, s: 45000, x: "living", b: "niger" },
                                { n: "Fula", d: 1000, s: 25000, x: "living", b: "niger", i: "West Africa" },
                                { n: "Wolof", d: 1000, s: 5500, x: "living", b: "niger", i: "Senegal" },
                                { n: "Akan", d: 1000, s: 11000, x: "living", b: "niger", i: "Ghana" }
                            ]
                        }
                    ]
                },

                // AUSTRONESIAN
                {
                    n: "Austronesian", d: -5000, b: "austro", i: "400M speakers, Pacific & SE Asia",
                    c: [
                        {
                            n: "Malayo-Polynesian", d: -4000, b: "austro", i: "Most Austronesian langs", c: [
                                { n: "Indonesian", d: 1900, s: 43000, x: "living", b: "austro", i: "200M+ L2" },
                                { n: "Malay", d: 500, s: 77000, x: "living", b: "austro" },
                                { n: "Javanese", d: 800, s: 82000, x: "living", b: "austro", i: "Java, Indonesia" },
                                { n: "Tagalog", d: 900, s: 28000, x: "living", b: "austro", i: "Philippines" },
                                { n: "Cebuano", d: 900, s: 21000, x: "living", b: "austro" },
                                { n: "Malagasy", d: 500, s: 25000, x: "living", b: "austro", i: "Madagascar" },
                                { n: "Maori", d: 1300, s: 150, x: "living", b: "austro", i: "New Zealand" },
                                { n: "Hawaiian", d: 1000, s: 24, x: "living", b: "austro" },
                                { n: "Samoan", d: 1000, s: 510, x: "living", b: "austro" },
                                { n: "Tongan", d: 1000, s: 187, x: "living", b: "austro" }
                            ]
                        },
                        {
                            n: "Formosan", d: -4500, b: "austro", i: "Taiwan, most diverse", c: [
                                { n: "Amis", d: 500, s: 200, x: "living", b: "austro" },
                                { n: "Paiwan", d: 500, s: 66, x: "living", b: "austro" }
                            ]
                        }
                    ]
                },

                // DRAVIDIAN
                {
                    n: "Dravidian", d: -4500, b: "dravid", i: "280M speakers, South India",
                    c: [
                        { n: "Telugu", d: 500, s: 83000, x: "living", b: "dravid", i: "83M native" },
                        { n: "Tamil", d: -300, s: 78000, x: "living", b: "dravid", i: "Ancient literary tradition" },
                        { n: "Kannada", d: 450, s: 44000, x: "living", b: "dravid" },
                        { n: "Malayalam", d: 830, s: 38000, x: "living", b: "dravid" },
                        { n: "Brahui", d: 1000, s: 2800, x: "living", b: "dravid", i: "Pakistan, isolated" }
                    ]
                },

                // TURKIC
                {
                    n: "Turkic", d: -2500, b: "turkic", i: "200M speakers, Central Asia to Turkey",
                    c: [
                        { n: "Turkish", d: 1000, s: 82000, x: "living", b: "turkic", i: "82M native" },
                        { n: "Azerbaijani", d: 1100, s: 23000, x: "living", b: "turkic" },
                        { n: "Uzbek", d: 1400, s: 27000, x: "living", b: "turkic" },
                        { n: "Kazakh", d: 1400, s: 13000, x: "living", b: "turkic" },
                        { n: "Uyghur", d: 800, s: 10000, x: "living", b: "turkic" },
                        { n: "Turkmen", d: 1000, s: 7000, x: "living", b: "turkic" },
                        { n: "Kyrgyz", d: 1500, s: 4500, x: "living", b: "turkic" },
                        { n: "Tatar", d: 1300, s: 5200, x: "living", b: "turkic" }
                    ]
                },

                // URALIC
                {
                    n: "Uralic", d: -6000, b: "uralic", i: "25M speakers",
                    c: [
                        {
                            n: "Finnic", d: -2000, b: "uralic", c: [
                                { n: "Finnish", d: 1500, s: 5400, x: "living", b: "uralic" },
                                { n: "Estonian", d: 1500, s: 1100, x: "living", b: "uralic" }
                            ]
                        },
                        {
                            n: "Ugric", d: -3000, b: "uralic", c: [
                                { n: "Hungarian", d: 896, s: 13000, x: "living", b: "uralic", i: "Unique in Central Europe" }
                            ]
                        },
                        {
                            n: "Sami", d: 500, b: "uralic", c: [
                                { n: "Northern Sami", d: 1000, s: 25, x: "living", b: "uralic" }
                            ]
                        }
                    ]
                },

                // JAPONIC
                {
                    n: "Japonic", d: -2000, b: "japonic", i: "128M speakers",
                    c: [
                        { n: "Japanese", d: 700, s: 125000, x: "living", b: "japonic", i: "125M speakers" },
                        { n: "Ryukyuan", d: 1100, s: 1000, x: "living", b: "japonic", i: "Okinawa, endangered" }
                    ]
                },

                // KOREANIC
                {
                    n: "Koreanic", d: -500, b: "koreanic", i: "80M speakers",
                    c: [
                        { n: "Korean", d: 600, s: 80000, x: "living", b: "koreanic", i: "80M, possible Altaic link" },
                        { n: "Jeju", d: 1000, s: 10, x: "living", b: "koreanic", i: "Critically endangered" }
                    ]
                },

                // TAI-KADAI
                {
                    n: "Tai-Kadai", d: -3000, b: "tai", i: "100M speakers, SE Asia",
                    c: [
                        { n: "Thai", d: 1200, s: 21000, x: "living", b: "tai", i: "60M total" },
                        { n: "Lao", d: 1200, s: 7000, x: "living", b: "tai" },
                        { n: "Shan", d: 1000, s: 3300, x: "living", b: "tai" },
                        { n: "Zhuang", d: 500, s: 16000, x: "living", b: "tai", i: "China's largest minority" }
                    ]
                },

                // AUSTROASIATIC
                {
                    n: "Austroasiatic", d: -4000, b: "austroasiatic", i: "117M speakers",
                    c: [
                        {
                            n: "Vietic", d: -1000, b: "austroasiatic", c: [
                                { n: "Vietnamese", d: 900, s: 85000, x: "living", b: "austroasiatic", i: "85M native" }
                            ]
                        },
                        { n: "Khmer", d: 600, s: 16000, x: "living", b: "austroasiatic", i: "Cambodia" },
                        { n: "Mon", d: 500, s: 850, x: "living", b: "austroasiatic" },
                        { n: "Santali", d: 500, s: 7600, x: "living", b: "austroasiatic", i: "India" }
                    ]
                },

                // KARTVELIAN
                {
                    n: "Kartvelian", d: -4000, b: "kartvelian", i: "South Caucasus",
                    c: [
                        { n: "Georgian", d: 400, s: 3700, x: "living", b: "kartvelian", i: "Unique alphabet" },
                        { n: "Mingrelian", d: 500, s: 340, x: "living", b: "kartvelian" },
                        { n: "Svan", d: 500, s: 15, x: "living", b: "kartvelian" }
                    ]
                },

                // MONGOLIC
                {
                    n: "Mongolic", d: -2000, b: "mongolic", i: "6M speakers",
                    c: [
                        { n: "Mongolian", d: 1200, s: 5700, x: "living", b: "mongolic" },
                        { n: "Buryat", d: 1200, s: 260, x: "living", b: "mongolic" },
                        { n: "Kalmyk", d: 1600, s: 80, x: "living", b: "mongolic" }
                    ]
                },

                // ISOLATES & SMALL FAMILIES
                {
                    n: "Isolates", d: -10000, b: "isolate", i: "No known relatives",
                    c: [
                        { n: "Basque", d: -500, s: 750, x: "living", b: "isolate", i: "Pre-IE European" },
                        { n: "Ainu", d: 500, s: 0.01, x: "living", b: "isolate", i: "Japan, nearly extinct" },
                        { n: "Burushaski", d: 500, s: 112, x: "living", b: "isolate", i: "Pakistan" },
                        { n: "Nihali", d: 500, s: 5, x: "living", b: "isolate", i: "India" }
                    ]
                },

                // AMERICAS
                {
                    n: "Americas", d: -15000, b: "americas", i: "Indigenous American families",
                    c: [
                        {
                            n: "Uto-Aztecan", d: -4000, b: "americas", c: [
                                { n: "Nahuatl", d: 600, s: 1700, x: "living", b: "americas", i: "Aztec language" }
                            ]
                        },
                        {
                            n: "Mayan", d: -4000, b: "americas", i: "30 languages", c: [
                                { n: "K'iche'", d: 500, s: 1000, x: "living", b: "americas" },
                                { n: "Yucatec Maya", d: 500, s: 800, x: "living", b: "americas" }
                            ]
                        },
                        {
                            n: "Quechuan", d: -2000, b: "americas", i: "Inca language family", c: [
                                { n: "Quechua", d: 500, s: 8900, x: "living", b: "americas", i: "8.9M in Andes" }
                            ]
                        },
                        {
                            n: "Aymaran", d: -2000, b: "americas", c: [
                                { n: "Aymara", d: 500, s: 1700, x: "living", b: "americas", i: "Bolivia, Peru" }
                            ]
                        },
                        {
                            n: "Algonquian", d: -3000, b: "americas", c: [
                                { n: "Cree", d: 1000, s: 96, x: "living", b: "americas" },
                                { n: "Ojibwe", d: 1000, s: 56, x: "living", b: "americas" }
                            ]
                        },
                        {
                            n: "Na-Dené", d: -10000, b: "americas", c: [
                                { n: "Navajo", d: 1000, s: 170, x: "living", b: "americas", i: "Largest US native lang" }
                            ]
                        },
                        {
                            n: "Eskimo-Aleut", d: -4000, b: "americas", c: [
                                { n: "Inuktitut", d: 1000, s: 39, x: "living", b: "americas" },
                                { n: "Greenlandic", d: 1000, s: 57, x: "living", b: "americas" }
                            ]
                        },
                        {
                            n: "Tupian", d: -3000, b: "americas", c: [
                                { n: "Guaraní", d: 500, s: 6500, x: "living", b: "americas", i: "Paraguay co-official" }
                            ]
                        }
                    ]
                },

                // AUSTRALIAN
                {
                    n: "Australian", d: -50000, b: "australian", i: "250+ language families",
                    c: [
                        {
                            n: "Pama-Nyungan", d: -6000, b: "australian", i: "Largest Australian family", c: [
                                { n: "Warlpiri", d: 500, s: 2.5, x: "living", b: "australian" },
                                { n: "Western Desert", d: 500, s: 7, x: "living", b: "australian" },
                                { n: "Arrernte", d: 500, s: 4.5, x: "living", b: "australian" }
                            ]
                        },
                        {
                            n: "Non-Pama-Nyungan", d: -40000, b: "australian", i: "Northern Australia", c: [
                                { n: "Tiwi", d: 500, s: 2, x: "living", b: "australian" }
                            ]
                        }
                    ]
                },

                // PAPUAN
                {
                    n: "Papuan", d: -40000, b: "papuan", i: "800+ languages, New Guinea",
                    c: [
                        {
                            n: "Trans-New Guinea", d: -10000, b: "papuan", i: "Largest Papuan group", c: [
                                { n: "Enga", d: 500, s: 230, x: "living", b: "papuan" },
                                { n: "Huli", d: 500, s: 150, x: "living", b: "papuan" }
                            ]
                        },
                        {
                            n: "Sepik", d: -10000, b: "papuan", c: [
                                { n: "Iatmul", d: 500, s: 10, x: "living", b: "papuan" }
                            ]
                        }
                    ]
                },

                // NILO-SAHARAN
                {
                    n: "Nilo-Saharan", d: -12000, b: "nilo", i: "50M speakers, Africa",
                    c: [
                        {
                            n: "Nilotic", d: -3000, b: "nilo", c: [
                                { n: "Dinka", d: 500, s: 1400, x: "living", b: "nilo", i: "South Sudan" },
                                { n: "Luo", d: 500, s: 6000, x: "living", b: "nilo", i: "Kenya, Tanzania" },
                                { n: "Maasai", d: 500, s: 1200, x: "living", b: "nilo" }
                            ]
                        },
                        {
                            n: "Saharan", d: -5000, b: "nilo", c: [
                                { n: "Kanuri", d: 1000, s: 4000, x: "living", b: "nilo", i: "Nigeria, Niger, Chad" }
                            ]
                        }
                    ]
                },

                // KHOISAN
                {
                    n: "Khoisan", d: -50000, b: "khoisan", i: "Click consonants, ancient",
                    c: [
                        {
                            n: "Khoe-Kwadi", d: -15000, b: "khoisan", c: [
                                { n: "Khoekhoe", d: 500, s: 200, x: "living", b: "khoisan", i: "Namibia" }
                            ]
                        },
                        {
                            n: "Tuu", d: -20000, b: "khoisan", c: [
                                { n: "ǃXóõ", d: 500, s: 4, x: "living", b: "khoisan", i: "Most phonemes of any language" }
                            ]
                        },
                        {
                            n: "Kx'a", d: -20000, b: "khoisan", c: [
                                { n: "ǂʼAmkoe", d: 500, s: 1.5, x: "living", b: "khoisan" }
                            ]
                        },
                        { n: "Hadza", d: -50000, s: 1, x: "living", b: "khoisan", i: "Tanzania, language isolate" },
                        { n: "Sandawe", d: -50000, s: 60, x: "living", b: "khoisan", i: "Tanzania isolate" }
                    ]
                },

                // NORTHEAST CAUCASIAN
                {
                    n: "NE Caucasian", d: -6000, b: "necaucas", i: "Dagestan, Chechnya",
                    c: [
                        { n: "Chechen", d: 1000, s: 1400, x: "living", b: "necaucas" },
                        { n: "Avar", d: 1000, s: 760, x: "living", b: "necaucas" },
                        { n: "Lezgian", d: 1000, s: 400, x: "living", b: "necaucas" }
                    ]
                },

                // NORTHWEST CAUCASIAN
                {
                    n: "NW Caucasian", d: -5000, b: "nwcaucas", i: "Complex consonant systems",
                    c: [
                        { n: "Circassian", d: 1000, s: 1000, x: "living", b: "nwcaucas" },
                        { n: "Abkhaz", d: 1000, s: 190, x: "living", b: "nwcaucas" }
                    ]
                },

                // HMONG-MIEN
                {
                    n: "Hmong-Mien", d: -4000, b: "hmong", i: "SE Asia highlands",
                    c: [
                        { n: "Hmong", d: 500, s: 4000, x: "living", b: "hmong", i: "Diaspora worldwide" },
                        { n: "Mien", d: 500, s: 1400, x: "living", b: "hmong" }
                    ]
                },

                // SIGN LANGUAGES
                {
                    n: "Sign Languages", d: -500, b: "sign", i: "Independent visual languages",
                    c: [
                        { n: "ASL", d: 1817, s: 500, x: "living", b: "sign", i: "American Sign Language" },
                        { n: "BSL", d: 1760, s: 150, x: "living", b: "sign", i: "British Sign Language" },
                        { n: "LSF", d: 1760, s: 100, x: "living", b: "sign", i: "French Sign, parent of many" },
                        { n: "CSL", d: 1950, s: 1000, x: "living", b: "sign", i: "Chinese Sign Language" }
                    ]
                }
            ]

        };

        const C = {
            ie: '#18FFFF',          // Cyan
            sino: '#FF6E40',        // Deep Orange
            afro: '#FFD740',        // Amber
            niger: '#FF4081',       // Pink
            austro: '#00E5FF',      // Cyan Accent
            dravid: '#EA80FC',      // Purple
            turkic: '#FFAB40',      // Orange
            uralic: '#B388FF',      // Deep Purple
            japonic: '#F48FB1',     // Pink Light
            koreanic: '#CE93D8',    // Purple Light
            tai: '#AED581',         // Light Green
            austroasiatic: '#80CBC4', // Teal Light
            nilo: '#64FFDA',        // Teal Accent
            isolate: '#90A4AE',     // Blue Grey
            kartvelian: '#FF8A65',  // Deep Orange Light
            mongolic: '#DCE775',    // Lime
            americas: '#4DB6AC',    // Teal
            australian: '#F06292',  // Pink Light
            papuan: '#9575CD',      // Deep Purple Light
            khoisan: '#FFF176',     // Yellow
            necaucas: '#7986CB',    // Indigo Light
            nwcaucas: '#9575CD',    // Deep Purple Light
            hmong: '#26A69A',       // Teal
            sign: '#42A5F5',        // Blue
            root: '#ffffff'
        };

        let W, H, PAD = 40, MINY = -100000, MAXY = 2025;
        const MAX_SPEAKERS = 2000000;
        const MIN_FAMILY_WIDTH = 120; // Minimum horizontal space per top-level family

        // Reorder children so largest (by speaker count) are in the center
        function centerLargestChildren(children) {
            if (!children || children.length < 2) return children;

            // Sort by total speakers descending
            let sorted = [...children].sort((a, b) => (b.ts || b.s || 0) - (a.ts || a.s || 0));

            // Interleave: largest in center, alternating left/right
            let result = new Array(sorted.length);
            let center = Math.floor(sorted.length / 2);
            let left = center, right = center + 1;

            for (let i = 0; i < sorted.length; i++) {
                if (i % 2 === 0) {
                    result[left--] = sorted[i];
                } else {
                    result[right++] = sorted[i];
                }
            }

            return result;
        }

        // Calculate content width based on visible nodes and their branches
        function calcContentWidth() {
            // Count all visible leaf nodes (end branches that need horizontal space)
            let leafCount = 0;
            const countVisibleLeaves = (n) => {
                if (!n.visible) return;
                if (!n.c || n.c.length === 0 || !n.expanded) {
                    leafCount++;
                } else {
                    n.c.forEach(countVisibleLeaves);
                }
            };
            if (D.c) D.c.forEach(countVisibleLeaves);

            // Each leaf needs minimum space, plus base width
            let leafWidth = leafCount * 60;
            let baseWidth = window.innerWidth * 1.2;
            return Math.max(leafWidth, baseWidth, window.innerWidth);
        }

        // Piecewise time scale - different rates for different eras
        // This gives 60% of vertical space to last 5000 years, where most languages are
        function timeToY(year) {
            const present = 2025;

            // Define time segments and their vertical allocation
            // Format: [startYear, endYear, startPercent, endPercent]
            const segments = [
                [2025, 1000, 0, 0.15],      // Present to 1000 CE: 15%
                [1000, 0, 0.15, 0.25],       // 1000 CE to 0: 10%
                [0, -2000, 0.25, 0.45],      // 0 to 2000 BCE: 20%
                [-2000, -5000, 0.45, 0.65],  // 2000-5000 BCE: 20%
                [-5000, -15000, 0.65, 0.80], // 5000-15000 BCE: 15%
                [-15000, -100000, 0.80, 1.0] // 15000-100000 BCE: 20%
            ];

            for (let [start, end, pStart, pEnd] of segments) {
                if (year <= start && year > end) {
                    let t = (start - year) / (start - end);
                    let pct = pStart + t * (pEnd - pStart);
                    return PAD + pct * (H - 2 * PAD);
                }
            }

            // Fallback for edge cases
            if (year > 2025) return PAD;
            return H - PAD;

        }
        let cv, ctx, nodes = [], drag = false, lx, ly, hover = null;
        // Camera/viewport system - proper pan/zoom model
        let camera = { x: 0, y: 0, zoom: 1 };
        let contentBounds = { minX: 0, maxX: 0, minY: 0, maxY: 0 };
        let animations = [];

        // Touch state for mobile
        let touchState = {
            active: false,
            touches: [],
            lastPinchDist: 0,
            lastTouchY: 0,
            lastTouchX: 0,
            isPinching: false
        };

        function proc(o, p = null, dep = 0) {
            o.p = p;
            o.dep = dep;
            o.s = o.s || 0;
            o.expanded = false;
            o.visible = dep === 0;
            o.growProgress = dep === 0 ? 1 : 0;
            let t = o.s;
            if (o.c) o.c.forEach(c => t += proc(c, o, dep + 1));
            o.ts = t;
            o.ey = o.x === 'living' ? MAXY : (o.e || (o.c?.length ? Math.max(...o.c.map(c => c.d)) : o.d + 200));
            o.hasChildren = o.c && o.c.length > 0;
            return t;
        }

        function layout(o, x0, x1) {
            // Store branch point for living languages
            o.branchY = timeToY(o.d);

            // Living end nodes (no children) positioned at present day
            if (o.x === 'living' && (!o.c || o.c.length === 0)) {
                o.y = timeToY(MAXY);  // Present day
            } else {
                o.y = timeToY(o.d);
            }

            o.yEnd = timeToY(o.ey);
            if (!o.c?.length) { o.px = (x0 + x1) / 2; return; }

            // Only allocate horizontal space for visible children
            let visibleChildren = o.c.filter(c => c.visible);
            if (visibleChildren.length === 0) {
                o.px = (x0 + x1) / 2;
                // Still need to layout invisible children (they need positions for when they become visible)
                o.c.forEach(c => layout(c, x0, x1));
                return;
            }

            // For root's children (top-level families), allocate width based on branch count
            // Reorder so largest families are in the center
            if (!o.p) {
                let orderedChildren = centerLargestChildren(visibleChildren);
                // Count descendants for each family to determine width allocation
                const countDescendants = (n) => {
                    if (!n.c) return 1;
                    return 1 + n.c.reduce((sum, child) => sum + countDescendants(child), 0);
                };
                // Calculate weight for each family (minimum weight ensures small families aren't invisible)
                let weights = orderedChildren.map(c => Math.max(3, Math.sqrt(countDescendants(c))));
                let totalWeight = weights.reduce((a, b) => a + b, 0);
                let cx = x0;
                orderedChildren.forEach((c, i) => {
                    let w = (weights[i] / totalWeight) * (x1 - x0);
                    layout(c, cx, cx + w);
                    cx += w;
                });
            } else {
                // For deeper levels, weight by speaker count
                // Give extra weight to living end nodes to space them out at present day
                let tw = visibleChildren.reduce((a, c) => {
                    let baseWeight = Math.sqrt((c.ts || 1) + 20);
                    let isLivingEnd = c.x === 'living' && (!c.c || c.c.length === 0);
                    return a + baseWeight * (isLivingEnd ? 2.5 : 1);  // 2.5x weight for living end nodes
                }, 0);

                let cx = x0;
                visibleChildren.forEach(c => {
                    let baseWeight = Math.sqrt((c.ts || 1) + 20);
                    let isLivingEnd = c.x === 'living' && (!c.c || c.c.length === 0);
                    let weight = baseWeight * (isLivingEnd ? 2.5 : 1);
                    let w = (weight / tw) * (x1 - x0);
                    layout(c, cx, cx + w);
                    cx += w;
                });
            }

            // Layout invisible children centered (for animation)
            let invisibleChildren = o.c.filter(c => !c.visible);
            invisibleChildren.forEach(c => layout(c, x0, x1));

            let xs = visibleChildren.map(c => c.px);
            o.px = (Math.min(...xs) + Math.max(...xs)) / 2;

        }

        function flat(o) { nodes.push(o); if (o.c) o.c.forEach(flat); }

        function getLineWidth(n) {
            let speakers = n.ts || n.s || 1;
            return Math.max(3, Math.pow(speakers / MAX_SPEAKERS, 0.35) * 40);
        }

        // Calculate actual bounds of visible content
        function updateContentBounds() {
            contentBounds = { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity };
            nodes.forEach(n => {
                if (n.visible) {
                    contentBounds.minX = Math.min(contentBounds.minX, n.px);
                    contentBounds.maxX = Math.max(contentBounds.maxX, n.px);
                    contentBounds.minY = Math.min(contentBounds.minY, n.y, n.yEnd || n.y);
                    contentBounds.maxY = Math.max(contentBounds.maxY, n.y, n.yEnd || n.y);
                }
            });
            // Always include root
            contentBounds.minY = Math.min(contentBounds.minY, D.y);
            contentBounds.maxY = Math.max(contentBounds.maxY, D.y);
            // Fallback if no visible nodes
            if (!isFinite(contentBounds.minX)) {
                contentBounds = { minX: 0, maxX: W, minY: 0, maxY: H };
            }
            // Add padding
            contentBounds.minX -= 50;
            contentBounds.maxX += 50;
            contentBounds.minY -= 50;
            contentBounds.maxY += 50;
        }

        // World to screen coordinate conversion
        function worldToScreen(wx, wy) {
            return {
                x: (wx - camera.x) * camera.zoom + cv.width / 2,
                y: (wy - camera.y) * camera.zoom + cv.height / 2
            };
        }

        // Screen to world coordinate conversion
        function screenToWorld(sx, sy) {
            return {
                x: (sx - cv.width / 2) / camera.zoom + camera.x,
                y: (sy - cv.height / 2) / camera.zoom + camera.y
            };
        }

        // Constrain camera to keep content in view
        function constrainCamera() {
            let contentW = contentBounds.maxX - contentBounds.minX;
            let contentH = contentBounds.maxY - contentBounds.minY;
            let contentCenterX = (contentBounds.minX + contentBounds.maxX) / 2;
            let contentCenterY = (contentBounds.minY + contentBounds.maxY) / 2;

            // At zoom 1, fit content width to viewport with some margin
            let viewW = cv.width / camera.zoom;
            let viewH = cv.height / camera.zoom;

            // Horizontal: allow panning but keep some content visible
            let minCamX = contentBounds.minX + viewW * 0.2;
            let maxCamX = contentBounds.maxX - viewW * 0.2;
            if (maxCamX < minCamX) {
                camera.x = contentCenterX;
            } else {
                camera.x = Math.max(minCamX, Math.min(maxCamX, camera.x));
            }

            // Vertical: constrain to content
            let minCamY = contentBounds.minY + viewH * 0.3;
            let maxCamY = contentBounds.maxY - viewH * 0.3;
            if (maxCamY < minCamY) {
                camera.y = contentCenterY;
            } else {
                camera.y = Math.max(minCamY, Math.min(maxCamY, camera.y));
            }

        }

        // Get full lineage - ancestors and descendants
        function getLineage(n) {
            let lineage = new Set();
            // Add ancestors
            let current = n;
            while (current) {
                lineage.add(current);
                current = current.p;
            }
            // Add descendants
            function addDescendants(node) {
                lineage.add(node);
                if (node.c) node.c.forEach(c => addDescendants(c));
            }
            addDescendants(n);
            return lineage;
        }

        // Check if point is near a bezier curve
        function pointNearBezier(mx, my, n) {
            if (!n.p || !n.visible || n.growProgress < 0.5) return false;
            let p = n.p;
            let midY = p.y + (n.y - p.y) * 0.5;
            let threshold = Math.max(15, getLineWidth(n) / 2 + 8);

            // Sample points along curve
            for (let t = 0; t <= 1; t += 0.05) {
                let mt = 1 - t;
                let x = mt * mt * mt * p.px + 3 * mt * mt * t * p.px + 3 * mt * t * t * n.px + t * t * t * n.px;
                let y = mt * mt * mt * p.y + 3 * mt * mt * t * midY + 3 * mt * t * t * midY + t * t * t * n.y;
                let dx = mx - x, dy = my - y;
                if (dx * dx + dy * dy < threshold * threshold) return true;
            }
            return false;

        }

        let highlightedLineage = null;

        function init() {
            cv = document.getElementById('canvas');
            ctx = cv.getContext('2d');  // Initialize rough.js
            cv.width = window.innerWidth;
            cv.height = window.innerHeight;
            H = Math.max(5000, cv.height * 5);
            PAD = 40;

            proc(D);

            // Mark root and first-level children as visible BEFORE layout
            D.expanded = true;
            D.visible = true;
            D.growProgress = 1;
            if (D.c) D.c.forEach(c => {
                c.visible = true;
                c.growProgress = 0;
            });

            // Calculate content width based on visible families
            W = calcContentWidth();
            layout(D, 10, W - 10);
            flat(D);
            updateContentBounds();

            // Position camera so root node is visible at bottom of screen
            // On mobile, zoom out to show more content
            let isMobile = window.innerWidth < 768;
            camera.zoom = isMobile ? 0.6 : 1;
            camera.x = (contentBounds.minX + contentBounds.maxX) / 2;
            // Camera.y is center of view. To show root at bottom, position camera so bottom of view is at maxY
            camera.y = contentBounds.maxY - (cv.height / camera.zoom) * 0.4;

            // Animate the children appearing
            if (D.c) D.c.forEach((c, i) => {
                setTimeout(() => animateNode(c), i * 80);
            });

            cv.addEventListener('wheel', e => {
                e.preventDefault();
                // Get world position under mouse before zoom
                let world = screenToWorld(e.clientX, e.clientY);

                // Adjust zoom
                let factor = e.deltaY > 0 ? 0.9 : 1.1;
                let newZoom = Math.max(0.15, Math.min(10, camera.zoom * factor));

                // Adjust camera so world point stays under mouse
                camera.zoom = newZoom;
                let newWorld = screenToWorld(e.clientX, e.clientY);
                camera.x += world.x - newWorld.x;
                camera.y += world.y - newWorld.y;

                constrainCamera();
                draw();
            }, { passive: false });

            cv.addEventListener('mousedown', e => { drag = true; lx = e.clientX; ly = e.clientY; });
            cv.addEventListener('mouseup', () => drag = false);
            cv.addEventListener('mouseleave', () => { drag = false; hover = null; highlightedLineage = null; hideTip(); draw(); });

            cv.addEventListener('mousemove', e => {
                if (drag) {
                    // Pan: move camera opposite to mouse movement
                    let dx = (e.clientX - lx) / camera.zoom;
                    let dy = (e.clientY - ly) / camera.zoom;
                    camera.x -= dx;
                    camera.y -= dy;
                    lx = e.clientX;
                    ly = e.clientY;
                    constrainCamera();
                    cv.style.cursor = 'grabbing';
                    draw();
                    return;
                }

                let world = screenToWorld(e.clientX, e.clientY);
                let mx = world.x, my = world.y;
                let found = null, minD = 2500;

                // Check nodes first
                nodes.forEach(n => {
                    if (!n.visible || n.growProgress < 0.5) return;
                    let dx = n.px - mx, dy = n.y - my, dd = dx * dx + dy * dy;
                    if (dd < minD) { minD = dd; found = n; }
                });

                // If not near a node, check connectors
                if (minD > 900) {
                    found = null;
                    for (let n of nodes) {
                        if (pointNearBezier(mx, my, n)) {
                            found = n;
                            break;
                        }
                    }
                }

                // Update highlight
                if (found !== hover) {
                    hover = found;
                    highlightedLineage = found ? getLineage(found) : null;
                    cv.style.cursor = hover ? (hover.hasChildren ? 'pointer' : 'default') : 'grab';
                    draw();
                    if (hover) showTip(e, hover);
                    else hideTip();
                } else if (hover) {
                    moveTip(e);
                }
            });

            cv.addEventListener('click', e => {
                if (hover && hover.hasChildren) {
                    if (hover.expanded) {
                        collapseNode(hover);
                    } else {
                        expandNode(hover);
                    }
                } else if (!hover) {
                    // Click on empty space clears highlight
                    highlightedLineage = null;
                    draw();
                }
            });

            // Mobile touch support
            cv.addEventListener('touchstart', e => {
                e.preventDefault();
                hideTip();
                touchState.active = true;
                touchState.touches = Array.from(e.touches);

                if (e.touches.length === 1) {
                    // Single finger - prepare for pan
                    touchState.isPinching = false;
                    touchState.lastTouchX = e.touches[0].clientX;
                    touchState.lastTouchY = e.touches[0].clientY;
                } else if (e.touches.length === 2) {
                    // Two fingers - prepare for pinch zoom
                    touchState.isPinching = true;
                    let dx = e.touches[0].clientX - e.touches[1].clientX;
                    let dy = e.touches[0].clientY - e.touches[1].clientY;
                    touchState.lastPinchDist = Math.sqrt(dx * dx + dy * dy);
                    // Store midpoint for zoom center
                    touchState.lastTouchX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
                    touchState.lastTouchY = (e.touches[0].clientY + e.touches[1].clientY) / 2;
                }
            }, { passive: false });

            cv.addEventListener('touchmove', e => {
                e.preventDefault();
                if (!touchState.active) return;

                if (e.touches.length === 1 && !touchState.isPinching) {
                    // Single finger pan (scroll)
                    let dx = (e.touches[0].clientX - touchState.lastTouchX) / camera.zoom;
                    let dy = (e.touches[0].clientY - touchState.lastTouchY) / camera.zoom;

                    camera.x -= dx;
                    camera.y -= dy;

                    touchState.lastTouchX = e.touches[0].clientX;
                    touchState.lastTouchY = e.touches[0].clientY;

                    constrainCamera();
                    draw();
                } else if (e.touches.length === 2) {
                    // Two finger pinch-to-zoom
                    touchState.isPinching = true;

                    let dx = e.touches[0].clientX - e.touches[1].clientX;
                    let dy = e.touches[0].clientY - e.touches[1].clientY;
                    let dist = Math.sqrt(dx * dx + dy * dy);

                    // Current midpoint
                    let midX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
                    let midY = (e.touches[0].clientY + e.touches[1].clientY) / 2;

                    if (touchState.lastPinchDist > 0) {
                        // Calculate zoom factor
                        let scale = dist / touchState.lastPinchDist;

                        // Get world position at midpoint before zoom
                        let world = screenToWorld(midX, midY);

                        // Apply zoom
                        let newZoom = Math.max(0.15, Math.min(10, camera.zoom * scale));
                        camera.zoom = newZoom;

                        // Adjust camera so world point stays at midpoint
                        let newWorld = screenToWorld(midX, midY);
                        camera.x += world.x - newWorld.x;
                        camera.y += world.y - newWorld.y;

                        // Also pan if midpoint moved
                        let panDx = (midX - touchState.lastTouchX) / camera.zoom;
                        let panDy = (midY - touchState.lastTouchY) / camera.zoom;
                        camera.x -= panDx;
                        camera.y -= panDy;

                        constrainCamera();
                        draw();
                    }

                    touchState.lastPinchDist = dist;
                    touchState.lastTouchX = midX;
                    touchState.lastTouchY = midY;
                }
            }, { passive: false });

            cv.addEventListener('touchend', e => {
                e.preventDefault();

                // Check for tap (click) on node
                if (touchState.touches.length === 1 && !touchState.isPinching && e.changedTouches.length === 1) {
                    let touch = e.changedTouches[0];
                    let world = screenToWorld(touch.clientX, touch.clientY);
                    let mx = world.x, my = world.y;

                    // Find closest node
                    let found = null, minD = 2500;
                    nodes.forEach(n => {
                        if (!n.visible || n.growProgress < 0.5) return;
                        let dx = n.px - mx, dy = n.y - my, dd = dx * dx + dy * dy;
                        if (dd < minD) { minD = dd; found = n; }
                    });

                    // If tap was on an expandable node
                    if (found && found.hasChildren && minD < 900) {
                        if (found.expanded) {
                            collapseNode(found);
                        } else {
                            expandNode(found);
                        }
                    }
                }

                // Reset touch state
                if (e.touches.length === 0) {
                    touchState.active = false;
                    touchState.isPinching = false;
                    touchState.lastPinchDist = 0;
                } else if (e.touches.length === 1) {
                    touchState.isPinching = false;
                    touchState.lastTouchX = e.touches[0].clientX;
                    touchState.lastTouchY = e.touches[0].clientY;
                }

                draw();
            }, { passive: false });

            cv.addEventListener('touchcancel', e => {
                touchState.active = false;
                touchState.isPinching = false;
                touchState.lastPinchDist = 0;
            });

            window.addEventListener('resize', resize);
            draw(); // Initial draw
            requestAnimationFrame(tick);

        }

        function animateNode(n) {
            animations.push({
                node: n,
                start: performance.now(),
                duration: 500
            });
        }

        function expandNode(n) {
            n.expanded = true;
            if (n.c) {
                n.c.forEach((c, i) => {
                    c.visible = true;
                    c.growProgress = 0;
                    setTimeout(() => animateNode(c), i * 50);
                });
            }
            W = calcContentWidth();
            layout(D, 10, W - 10);
            updateContentBounds();
        }

        function collapseNode(n) {
            n.expanded = false;
            const hideAll = o => {
                if (o.c) o.c.forEach(c => {
                    c.visible = false;
                    c.expanded = false;
                    c.growProgress = 0;
                    hideAll(c);
                });
            };
            hideAll(n);
            W = calcContentWidth();
            layout(D, 10, W - 10);
            updateContentBounds();
        }

        function expandAll() {
            const expandR = o => {
                if (o.c && o.c.length) {
                    o.expanded = true;
                    o.c.forEach(c => {
                        c.visible = true;
                        if (c.growProgress < 1) animateNode(c);
                        expandR(c);
                    });
                }
            };
            expandR(D);
            W = calcContentWidth();
            layout(D, 10, W - 10);
            updateContentBounds();
        }

        function collapseAll() {
            const collapseR = o => {
                o.expanded = false;
                if (o.c) o.c.forEach(c => {
                    c.visible = o === D;
                    c.growProgress = o === D ? 1 : 0;
                    c.expanded = false;
                    collapseR(c);
                });
            };
            collapseR(D);
            D.expanded = true;
            if (D.c) D.c.forEach(c => { c.visible = true; c.growProgress = 1; });
            W = calcContentWidth();
            layout(D, 10, W - 10);
            updateContentBounds();
            // Show family nodes
            let isMobile = window.innerWidth < 768;
            camera.zoom = isMobile ? 0.5 : 0.8;
            camera.x = (contentBounds.minX + contentBounds.maxX) / 2;
            camera.y = contentBounds.minY + (cv.height / camera.zoom) * 0.5;
            draw();
        }

        function resize() {
            cv.width = window.innerWidth;
            cv.height = window.innerHeight;
            // Tall virtual height for time scale
            H = Math.max(5000, cv.height * 5);
            PAD = 40;

            if (nodes.length > 0) {
                W = calcContentWidth();
                layout(D, 10, W - 10);
                updateContentBounds();
                // Keep current zoom but recenter
                camera.x = (contentBounds.minX + contentBounds.maxX) / 2;
                constrainCamera();
            }
            draw();

        }

        function reset() {
            H = Math.max(5000, cv.height * 5);
            PAD = 40;

            W = calcContentWidth();
            layout(D, 10, W - 10);
            updateContentBounds();
            let isMobile = window.innerWidth < 768;
            camera.zoom = isMobile ? 0.6 : 1;
            camera.x = (contentBounds.minX + contentBounds.maxX) / 2;
            camera.y = contentBounds.maxY - (cv.height / camera.zoom) * 0.4;
            constrainCamera();
            draw();

        }

        function tick() {
            let now = performance.now();
            let needsDraw = false;

            animations = animations.filter(a => {
                let t = (now - a.start) / a.duration;
                if (t >= 1) {
                    a.node.growProgress = 1;
                    needsDraw = true;
                    return false;
                }
                a.node.growProgress = 1 - Math.pow(1 - t, 3);
                needsDraw = true;
                return true;
            });

            if (needsDraw || animations.length > 0) draw();
            requestAnimationFrame(tick);

        }

        function draw() {
            ctx.setTransform(1, 0, 0, 1, 0, 0);
            ctx.fillStyle = '#030508';
            ctx.fillRect(0, 0, cv.width, cv.height);

            // Camera transform: translate so camera position is at screen center, then scale
            ctx.setTransform(
                camera.zoom, 0, 0, camera.zoom,
                cv.width / 2 - camera.x * camera.zoom,
                cv.height / 2 - camera.y * camera.zoom
            );

            // Time axis labels
            ctx.strokeStyle = 'rgba(255,255,255,0.05)';
            ctx.lineWidth = 1;
            ctx.fillStyle = '#3a4550';
            ctx.font = '11px system-ui';
            [2000, 1500, 1000, 500, 0, -500, -1000, -2000, -3000, -5000, -10000, -50000].forEach(yr => {
                let y = timeToY(yr);
                ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(W, y); ctx.stroke();
                let label = yr < 0 ? Math.abs(yr) >= 1000 ? (Math.abs(yr) / 1000) + 'k BCE' : Math.abs(yr) + ' BCE' : yr + ' CE';
                ctx.fillText(label, 15, y - 10);
            });

            // Present line
            let presY = timeToY(2025);
            ctx.strokeStyle = 'rgba(255,255,255,0.15)';
            ctx.lineWidth = 2;
            ctx.setLineDash([8, 12]);
            ctx.beginPath(); ctx.moveTo(0, presY); ctx.lineTo(W, presY); ctx.stroke();
            ctx.setLineDash([]);
            ctx.fillStyle = '#64748b';
            ctx.font = '600 11px system-ui';
            ctx.textAlign = 'center';
            ctx.fillText('PRESENT', W / 2, presY - 12);
            ctx.textAlign = 'left';

            // Draw lines
            nodes.forEach(n => {
                if (!n.visible || n.growProgress === 0) return;

                let col = C[n.b] || '#666';
                let lw = getLineWidth(n);
                let prog = n.growProgress;

                // Check if this node is in the highlighted lineage
                let inLineage = !highlightedLineage || highlightedLineage.has(n);
                let lineAlpha = inLineage ? 1 : 0.08;

                // Branch from parent
                if (n.p) {
                    let p = n.p;
                    let isLivingEndNode = n.x === 'living' && (!n.c || n.c.length === 0);

                    // For living end nodes, draw to branch point instead of present
                    let targetY = isLivingEndNode ? n.branchY : n.y;
                    let midY = p.y + (targetY - p.y) * 0.5;

                    let alpha = (n.x === 'extinct' ? 0.25 : 0.5) * prog * lineAlpha;
                    ctx.globalAlpha = alpha;

                    // Smooth bezier curve with glow
                    ctx.strokeStyle = col;
                    ctx.lineWidth = lw;
                    ctx.lineCap = 'round';
                    ctx.lineJoin = 'round';

                    // Add glow effect
                    if (inLineage) {
                        ctx.shadowColor = col;
                        ctx.shadowBlur = highlightedLineage ? 15 : 8;
                    }

                    // Draw smooth bezier curve
                    ctx.beginPath();
                    ctx.moveTo(p.px, p.y);
                    ctx.bezierCurveTo(p.px, midY, n.px, midY, n.px, targetY * prog + p.y * (1 - prog));
                    ctx.stroke();

                    ctx.shadowBlur = 0;

                    // For living end nodes, draw line from branch point down to present (same color/width as parent)
                    if (isLivingEndNode && prog > 0.8) {
                        let contProg = (prog - 0.8) / 0.2;
                        ctx.globalAlpha = (n.x === 'extinct' ? 0.25 : 0.5) * contProg * lineAlpha;

                        // Draw continuation line smooth
                        let endY = n.branchY + (n.y - n.branchY) * contProg;
                        ctx.strokeStyle = col;
                        ctx.lineWidth = lw;
                        ctx.lineCap = 'round';
                        if (inLineage) {
                            ctx.shadowColor = col;
                            ctx.shadowBlur = 8;
                        }
                        ctx.beginPath();
                        ctx.moveTo(n.px, n.branchY);
                        ctx.lineTo(n.px, endY);
                        ctx.stroke();
                        ctx.shadowBlur = 0;
                    }

                    // Extinct continuation
                    if (n.x === 'extinct' && n.yEnd < n.y && prog === 1) {
                        ctx.globalAlpha = 0.2 * lineAlpha;
                        ctx.strokeStyle = col;
                        ctx.lineWidth = lw * 0.4;
                        ctx.lineCap = 'round';
                        ctx.beginPath();
                        ctx.moveTo(n.px, n.y);
                        ctx.lineTo(n.px, n.yEnd);
                        ctx.stroke();
                    }
                }
            });

            // Draw nodes
            ctx.globalAlpha = 1;
            let now = performance.now();

            nodes.forEach(n => {
                if (!n.visible || n.growProgress < 0.3) return;

                let prog = Math.min(1, (n.growProgress - 0.3) / 0.7);
                let baseR = n.dep === 0 ? 32 : (n.hasChildren ? 22 : 16);
                let r = baseR * prog;

                // Check if in highlighted lineage
                let inLineage = !highlightedLineage || highlightedLineage.has(n);
                let nodeAlpha = inLineage ? 1 : 0.12;

                let isExpandable = n.hasChildren && !n.expanded && n.growProgress === 1;
                let pulse = isExpandable ? 1 + 0.05 * Math.sin(now / 300) : 1;

                // Base glow for all nodes
                let alpha = prog * (n.x === 'extinct' ? 0.5 : 1) * nodeAlpha;
                ctx.globalAlpha = alpha;

                let color = C[n.b] || '#666';

                // Glowing halo
                if (inLineage) {
                    ctx.shadowColor = color;
                    ctx.shadowBlur = isExpandable ? 20 + 8 * Math.sin(now / 300) : 12;
                }

                // Draw node circle
                ctx.fillStyle = color;
                ctx.beginPath();
                ctx.arc(n.px, n.y, r * pulse, 0, Math.PI * 2);
                ctx.fill();
                ctx.shadowBlur = 0;

                // Stroke for special states
                if (isExpandable) {
                    ctx.strokeStyle = '#3b82f6';
                    ctx.lineWidth = 2;
                    ctx.shadowColor = '#3b82f6';
                    ctx.shadowBlur = 10;
                    ctx.stroke();
                    ctx.shadowBlur = 0;
                } else if (n.hasChildren && n.expanded) {
                    ctx.strokeStyle = 'rgba(255,255,255,0.3)';
                    ctx.lineWidth = 1.5;
                    ctx.stroke();
                } else if (n.x === 'living') {
                    ctx.strokeStyle = 'rgba(255,255,255,0.4)';
                    ctx.lineWidth = 1.5;
                    ctx.stroke();
                }

                // Plus icon
                if (isExpandable && r > 8 && inLineage) {
                    ctx.strokeStyle = '#fff';
                    ctx.lineWidth = 2.5;
                    ctx.globalAlpha = 0.95;
                    let iconSize = r * 0.45;
                    ctx.beginPath();
                    ctx.moveTo(n.px - iconSize, n.y);
                    ctx.lineTo(n.px + iconSize, n.y);
                    ctx.moveTo(n.px, n.y - iconSize);
                    ctx.lineTo(n.px, n.y + iconSize);
                    ctx.stroke();
                }

                // Minus icon
                if (n.hasChildren && n.expanded && r > 8 && inLineage) {
                    ctx.strokeStyle = 'rgba(255,255,255,0.5)';
                    ctx.lineWidth = 2;
                    ctx.globalAlpha = 0.8;
                    let iconSize = r * 0.4;
                    ctx.beginPath();
                    ctx.moveTo(n.px - iconSize, n.y);
                    ctx.lineTo(n.px + iconSize, n.y);
                    ctx.stroke();
                }

                // Hover glow ring
                if (n === hover) {
                    ctx.strokeStyle = n.hasChildren ? '#60a5fa' : color;
                    ctx.lineWidth = 2;
                    ctx.globalAlpha = 1;
                    ctx.shadowColor = n.hasChildren ? '#60a5fa' : color;
                    ctx.shadowBlur = 20;
                    ctx.beginPath();
                    ctx.arc(n.px, n.y, r + 6, 0, Math.PI * 2);
                    ctx.stroke();
                    ctx.shadowBlur = 0;
                }

                // Label
                if (n.dep > 0 && prog > 0.5) {
                    ctx.globalAlpha = prog * (n.x === 'extinct' ? 0.4 : 0.7) * nodeAlpha;
                    ctx.fillStyle = '#94a3b8';
                    let fontSize = n.dep <= 1 ? 18 : 15;
                    ctx.font = (n.dep <= 1 || n.s > 50000 ? '600 ' : '500 ') + fontSize + 'px system-ui';

                    // For living end nodes at present, angle the labels
                    let isLivingEndNode = n.x === 'living' && (!n.c || n.c.length === 0);

                    if (isLivingEndNode) {
                        // Save context, rotate, and draw angled label
                        ctx.save();
                        ctx.translate(n.px, n.y);
                        ctx.rotate(-Math.PI / 4);  // -45 degrees
                        ctx.textAlign = 'left';
                        ctx.fillText(n.n, r + 8, 0);
                        ctx.restore();
                    } else {
                        // Normal horizontal labels for other nodes
                        ctx.textAlign = n.px > W / 2 ? 'right' : 'left';
                        let labelX = n.px + (n.px > W / 2 ? -r - 10 : r + 10);
                        ctx.fillText(n.n, labelX, n.y + 4);
                    }
                }
            });

            // Root node
            if (D.visible) {
                let inLineage = !highlightedLineage || highlightedLineage.has(D);
                ctx.globalAlpha = inLineage ? 1 : 0.15;

                // Root glow
                ctx.shadowColor = C.root;
                ctx.shadowBlur = 25;
                ctx.fillStyle = C.root;
                ctx.beginPath();
                ctx.arc(D.px, D.y, 36, 0, Math.PI * 2);
                ctx.fill();
                ctx.shadowBlur = 0;

                ctx.strokeStyle = 'rgba(255,255,255,0.4)';
                ctx.lineWidth = 2;
                ctx.stroke();

                // Root label
                ctx.fillStyle = '#94a3b8';
                ctx.font = '600 12px system-ui';
                ctx.textAlign = 'center';
                ctx.fillText('HUMAN LANGUAGES', D.px, D.y + 55);
            }

            ctx.textAlign = 'left';
            ctx.globalAlpha = 1;

        }

        function fmtSpk(s) {
            if (s >= 1000000) return (s / 1000000).toFixed(1) + 'B';
            if (s >= 1000) return Math.round(s / 1000) + 'M';
            if (s >= 1) return s + 'K';
            return '<1K';
        }

        function showTip(e, n) {
            let t = document.getElementById('tip');
            let h = '<h3 style="color:' + (C[n.b] || '#888') + '">' + n.n + '</h3>';
            h += '<div class="meta">' + (n.d < 0 ? Math.abs(n.d) >= 1000 ? (Math.abs(n.d) / 1000) + 'k BCE' : Math.abs(n.d) + ' BCE' : n.d + ' CE');
            h += n.x === 'living' ? ' → Present' : (n.e ? ' → ' + (n.e < 0 ? Math.abs(n.e) + ' BCE' : n.e + ' CE') : '');
            h += '</div>';
            if (n.s) h += '<div class="spk">' + fmtSpk(n.s) + ' speakers</div>';
            if (n.ts > n.s) h += '<div class="total">Family total: ' + fmtSpk(n.ts) + '</div>';
            if (n.i) h += '<p>' + n.i + '</p>';
            if (n.hasChildren) {
                if (n.expanded) {
                    h += '<div class="hint">➖ Click to collapse</div>';
                } else {
                    h += '<div class="hint">➕ Click to expand ' + n.c.length + ' branches</div>';
                }
            }
            t.innerHTML = h;
            t.style.left = Math.min(e.clientX + 15, window.innerWidth - 340) + 'px';
            t.style.top = Math.min(e.clientY + 15, window.innerHeight - 200) + 'px';
            t.classList.add('visible');
        }

        function moveTip(e) {
            let t = document.getElementById('tip');
            t.style.left = Math.min(e.clientX + 15, window.innerWidth - 340) + 'px';
            t.style.top = Math.min(e.clientY + 15, window.innerHeight - 200) + 'px';
        }

        function hideTip() { document.getElementById('tip').classList.remove('visible'); }

        init();