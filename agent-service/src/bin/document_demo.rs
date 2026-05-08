//! **Document Improvement Demo**
//!
//! Dit is een interactieve demo van de document agent pipeline.
//! Je kunt een document invoeren en kiezen uit verschillende verbeteringen.
//!
//! Run met: `cargo run --bin document_demo`

use std::collections::HashMap;
use std::io::{self, Write};

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};

use agent_service::mesh::expert::{
    ExpertMsg, PIICategory, ReviewCriteria, StyleProfile, WorkEnvelope,
    WorkPayload,
};
use agent_service::mesh::experts::{
    spawn_pii_stripper_expert, spawn_reviewer_expert, spawn_schrijver_expert,
};
use agent_service::mesh::types::SessionContext;

/// Voorbeeld document voor de demo
const SAMPLE_DOCUMENT: &str = r#"
# Project Update - Q1 2025

## Team
Contact: John Doe (john.doe@example.com) of Jane Smith (jane.smith@company.com)
Phone: +1-555-123-4567

## Voortgang
Het project loopt goed. We hebben veel gedaan.
TODO: Implementeer de nieuwe feature
TODO: Schrijf documentatie

Er zijn lange zinnen in dit document die eigenlijk te lang zijn voor goede leesbaarheid en dat moet eigenlijk beter kunnen omdat lange zinnen moeilijk te lezen zijn en de boodschap minder duidelijk maken.

## Financiën
IBAN: NL91ABNA0417164300
Het budget is overschreden.

## Conclusie
We gaan door met het project.
"#;

/// Verbeteringsopties
#[derive(Debug, Clone)]
enum ImprovementOption {
    Review,
    Summarize,
    Formalize,
    ScrubPII,
    Shorten,
    CorrectTone,
}

impl ImprovementOption {
    fn description(&self) -> &'static str {
        match self {
            Self::Review => "🔍 Review - Controleer op regellengte, TODO's, structuur",
            Self::Summarize => "📝 Samenvatten - Maak een korte samenvatting",
            Self::Formalize => "💼 Formaliseren - Maak de toon professioneler",
            Self::ScrubPII => "🔒 PII verwijderen - Verberg emails, namen, telefoonnummers",
            Self::Shorten => "✂️ Verkorten - Maak de tekst compacter",
            Self::CorrectTone => "✨ Toon corrigeren - Verbeter leesbaarheid en stijl",
        }
    }

    fn all() -> Vec<(String, Self)> {
        vec![
            ("1".into(), Self::Review),
            ("2".into(), Self::Summarize),
            ("3".into(), Self::Formalize),
            ("4".into(), Self::ScrubPII),
            ("5".into(), Self::Shorten),
            ("6".into(), Self::CorrectTone),
        ]
    }
}

struct CaptureReply {
    tx: Option<tokio::sync::oneshot::Sender<String>>,
}

#[async_trait::async_trait]
impl Actor for CaptureReply {
    type Msg = ExpertMsg;
    type State = CaptureReply;
    type Arguments = tokio::sync::oneshot::Sender<String>;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(CaptureReply { tx: Some(args) })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ExpertMsg::Work(envelope) => {
                if let Some(sender) = state.tx.take() {
                    let result = match &envelope.payload {
                        WorkPayload::Review { content, .. } => {
                            format!("REVIEW RESULT:\n\n{}", content)
                        }
                        WorkPayload::Write { research_notes, .. } => {
                            format!("WRITE RESULT:\n\n{}", research_notes)
                        }
                        WorkPayload::ScrubPII { content, .. } => {
                            format!("SCRUB RESULT:\n\n{}", content)
                        }
                        _ => "Onverwacht payload type".to_string(),
                    };
                    let _ = sender.send(result);
                }
            }
            ExpertMsg::PeerResponse { result, .. } => {
                if let Some(sender) = state.tx.take() {
                    let _ = sender.send(format!("PEER RESPONSE:\n\n{}", result));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Voer een verbetering uit via de pipeline
async fn apply_improvement(
    document: &str,
    option: &ImprovementOption,
    live_tx: Option<broadcast::Sender<agent_service::AgentEvent>>,
) -> Result<String, String> {
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let (capture_ref, _) = Actor::spawn(None, CaptureReply { tx: None }, tx)
        .await
        .map_err(|e| format!("capture spawn: {}", e))?;

    let envelope = match option {
        ImprovementOption::Review => WorkEnvelope {
            payload: WorkPayload::Review {
                content: document.to_string(),
                criteria: Some(ReviewCriteria {
                    tone: None,
                    length_constraints: Some((10, 1000)),
                    focus_areas: vec!["clarity".into(), "structure".into()],
                }),
            },
            context: SessionContext {
                session_id: uuid::Uuid::new_v4(),
                user_id: "demo".into(),
                metadata: HashMap::new(),
            },
            reply_to: Some(capture_ref.clone()),
            entry_reply: None,
            trace_id: uuid::Uuid::new_v4(),
            hop_count: 0,
        },
        ImprovementOption::ScrubPII => WorkEnvelope {
            payload: WorkPayload::ScrubPII {
                content: document.to_string(),
                pii_categories: vec![
                    PIICategory::Email,
                    PIICategory::PhoneNumber,
                    PIICategory::Name,
                    PIICategory::IBAN,
                ],
            },
            context: SessionContext {
                session_id: uuid::Uuid::new_v4(),
                user_id: "demo".into(),
                metadata: HashMap::new(),
            },
            reply_to: Some(capture_ref.clone()),
            entry_reply: None,
            trace_id: uuid::Uuid::new_v4(),
            hop_count: 0,
        },
        ImprovementOption::Summarize
        | ImprovementOption::Formalize
        | ImprovementOption::Shorten
        | ImprovementOption::CorrectTone => {
            // SchrijverExpert handles these via style profile
            let style = match option {
                ImprovementOption::Summarize => StyleProfile::Casual,
                ImprovementOption::Formalize => StyleProfile::Formal,
                ImprovementOption::Shorten => StyleProfile::Technical,
                ImprovementOption::CorrectTone => StyleProfile::Custom("corrigeer en verbeter de toon".into()),
                _ => StyleProfile::Formal,
            };

            let prompt = match option {
                ImprovementOption::Summarize => "Maak een samenvatting van de volgende tekst, max 10 regels:",
                ImprovementOption::Formalize => "Herschrijf de volgende tekst in een professionele, formele toon:",
                ImprovementOption::Shorten => "Verkort de volgende tekst tot de kern, max 100 woorden:",
                ImprovementOption::CorrectTone => "Verbeter de volgende tekst qua leesbaarheid, stijl en toon:",
                _ => "",
            };

            WorkEnvelope {
                payload: WorkPayload::Write {
                    research_notes: format!("{}\n\n{}", prompt, document),
                    style_profile: style,
                },
                context: SessionContext {
                    session_id: uuid::Uuid::new_v4(),
                    user_id: "demo".into(),
                    metadata: HashMap::new(),
                },
                reply_to: Some(capture_ref.clone()),
                entry_reply: None,
                trace_id: uuid::Uuid::new_v4(),
                hop_count: 0,
            }
        }
    };

    let expert = match option {
        ImprovementOption::Review => {
            spawn_reviewer_expert(live_tx.clone())
                .await
                .map_err(|e| format!("reviewer spawn: {}", e))?
        }
        ImprovementOption::ScrubPII => {
            spawn_pii_stripper_expert(live_tx.clone())
                .await
                .map_err(|e| format!("pii spawn: {}", e))?
        }
        _ => {
            spawn_schrijver_expert(live_tx.clone())
                .await
                .map_err(|e| format!("schrijver spawn: {}", e))?
        }
    };

    expert
        .cast(ExpertMsg::Work(envelope))
        .map_err(|e| format!("send work: {}", e))?;

    timeout(Duration::from_secs(90), rx)
        .await
        .map_err(|_| "Timeout na 90 seconden".into())?
        .map_err(|e| format!("recv error: {}", e))
}

fn print_header(title: &str) {
    println!("\n{}", "═".repeat(60));
    println!("  {}", title);
    println!("{}", "═".repeat(60));
}

fn print_document(label: &str, content: &str) {
    println!("\n📄 {}", label);
    println!("{}", "─".repeat(40));
    println!("{}", content);
}

async fn run_interactive_demo() -> Result<(), Box<dyn std::error::Error>> {
    let live_tx = if std::env::var("MESH_LIVE").is_ok() {
        Some(broadcast::channel(100).0)
    } else {
        None
    };

    print_header("Document Improvement Pipeline Demo");

    println!(
        "\nDit demonstreert de document verbetering pipeline met de volgende experts:"
    );
    println!("  • ReviewerExpert  - Review met inline annotaties");
    println!("  • SchrijverExpert - Document herschrijven (met Ollama indien beschikbaar)");
    println!("  • PIIStripperExpert - PII detectie en anonimisatie");

    // Start met voorbeeld document
    print_document("ORIGINEEL DOCUMENT", SAMPLE_DOCUMENT);

    println!("\n\n⚠️  Opmerking: SchrijverExpert gebruikt Ollama als MESH_USE_OLLAMA=1 is gezet.");
    println!("           Anders wordt een fallback samenvatting gebruikt.\n");

    // Main menu loop
    loop {
        println!("\n📋 Kies een verbetering (of 'q' om te stoppen):\n");

        for (key, option) in ImprovementOption::all() {
            println!("  [{}] {}", key, option.description());
        }
        println!("  [c]  🔄 Eigen document invoeren");
        println!("  [s]  📄 Toon huidig document");
        println!("  [q]  🚪 Stop\n");

        print!("Keuze: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "q" | "quit" | "exit" => {
                println!("\n👋 Bedankt voor het testen!");
                break;
            }
            "c" | "custom" => {
                print!("\nPlak je document (druk Enter 2x om te eindigen):\n");
                let mut custom_doc = String::new();
                loop {
                    let mut line = String::new();
                    io::stdin().read_line(&mut line)?;
                    if line.trim().is_empty() && !custom_doc.is_empty() {
                        break;
                    }
                    custom_doc.push_str(&line);
                }
                if !custom_doc.trim().is_empty() {
                    print_document("JOUW DOCUMENT", &custom_doc);
                    println!("\n💡 Tip: Kies nu een verbetering optie hierboven.");
                }
            }
            "s" | "show" => {
                // Shows the current document (would need state, for now shows sample)
                print_document("HUIDIG DOCUMENT", SAMPLE_DOCUMENT);
            }
            _ => {
                // Parse option
                let option = ImprovementOption::all()
                    .into_iter()
                    .find(|(k, _)| k == input)
                    .map(|(_, o)| o);

                if let Some(opt) = option {
                    print_header(&format!("Verwerken: {:?}", opt));

                    match apply_improvement(SAMPLE_DOCUMENT, &opt, live_tx.clone()).await {
                        Ok(result) => {
                            print_document("RESULTAAT", &result);
                        }
                        Err(e) => {
                            println!("❌ Fout: {}", e);
                        }
                    }
                } else {
                    println!("❌ Ongeldige keuze: {}", input);
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    run_interactive_demo().await
}
