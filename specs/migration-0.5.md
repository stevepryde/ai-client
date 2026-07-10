# ai-client 0.4 to 0.5 migration

Status: draft

## Native OpenAI Responses models are typed only

The native Responses API no longer exposes the dynamic OpenAI model/request
types, runtime capability catalogs, or validation modes. Those APIs allowed a
known model ID to bypass compile-time capability checks.

Applications that select a model at runtime should use a closed application
enum and match it to the corresponding typed builder. Custom aliases and
fine-tuned models use an application-defined model marker and explicitly
implement only the capability traits supported by that model.

## Reasoning and sampling are mutually typed

GPT-5.1 and GPT-5.4 accept sampling controls only with no reasoning. The builder
records that mode in its type. After selecting a non-none reasoning effort,
sampling methods do not exist. After a sampling method, non-none reasoning
methods do not exist. The separate reasoning-none method lets the compiler
enforce both call orders.
