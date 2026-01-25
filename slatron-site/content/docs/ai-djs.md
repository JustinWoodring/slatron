+++
title = "AI DJs & Orpheus"
description = "Configure intelligent hosts for your station."
weight = 2
+++

Slatron features a dynamic **AI DJ** system that can "speak" between songs, introducing tracks, reading news, or just creating a vibe.

## LLM Providers
You can configure different LLM providers for DJ personality:
*   **Google Gemini**: High-quality cloud-based models
*   **Anthropic Claude**: Claude 3.5 Sonnet and other Claude models
*   **OpenAI**: GPT-4 and other OpenAI models
*   **Ollama**: Local LLM server for privacy and cost savings

## Voice Providers
*   **Gemini TTS**: High-quality cloud-based TTS from Google
*   **Orpheus (Local)**: Fully offline, local text-to-speech engine powered by specific LLM checkpoints

## Setting up Orpheus (Local TTS)
1.  **Install LM Studio**.
2.  **Download Model**: `isaiahbjork/orpheus-3b-0.1-ft`.
3.  **Start Local Server** on port `1234`.
4.  **Configure Slatron**: Add provider type **Orpheus** with endpoint `http://127.0.0.1:1234/v1/completions`.
