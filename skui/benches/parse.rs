use criterion::{criterion_group, criterion_main, Criterion};
use skui::SKUI;

fn parse(src: &str) -> SKUI {
    SKUI::parse(src).unwrap()
}

fn bench_parse(c: &mut Criterion) {
    let input = r#"
            Flex { background-color: black; padding:1px }
            #list { border: 1px solid yellow }
            .myBtn { border: 2px }
            #myFlex { border:2px }
            .background_white { background-color: WHITE }

            Flex(MainFill) #myFlex .background_white {
                myProperty1 : "data"
                propertyMap : {key=1, key2=true}
                propertyAnother : [ 1,2,3 ]
                FlexItem(1.0, Button("FlexItem1"))
                FlexItem(2.0, Button("FlexItem2"))
                Button()
                Flex() {
                    Label("1") Label("2")
                }
            }

            Grid(2,3) {
                Label()
            }

            CustomWidget() {
                Flex() {
                    FlexItem( 0.5, Button("OK") )
                    FlexItem( 0.5, Button("Cancel") )
                }
            }
        "#;
    c.bench_function("parse", |b| {
        b.iter(|| parse(std::hint::black_box(input)) )
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);