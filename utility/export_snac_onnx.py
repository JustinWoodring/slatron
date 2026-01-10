#!/usr/bin/env python3
import torch
import sys
import os

try:
    from snac import SNAC
except ImportError:
    print("Error: 'snac' library not found. Please install it or run this in the orpheus-tts-local environment.", file=sys.stderr)
    sys.exit(1)

class SnacWrapper(torch.nn.Module):
    def __init__(self, model):
        super().__init__()
        self.model = model
        
    def forward(self, c0, c1, c2):
        # Wrapper to accept 3 tensors and return audio
        codes = [c0, c1, c2]
        return self.model.decode(codes)

def export_model(output_path="snac.onnx"):
    device = "cpu"
    print("Loading SNAC model...")
    model = SNAC.from_pretrained("hubertsiuzdak/snac_24khz").to(device)
    model.eval()
    
    wrapper = SnacWrapper(model)
    
    # Create dummy inputs
    # Layer 0: 12Hz (~1 token per frame roughly?) - Shape [B, T]
    # Layer 1: 24Hz
    # Layer 2: 48Hz
    # Wait, 24khz model has specific strides.
    # From paper/repo: 
    # codes are list of 3 tensors.
    # Let's use dummy inputs of length 1, 2, 4 respectively?
    # Actually SNAC 24khz usually has Fixed ratio.
    # The snippet "num_frames = len(multiframe) // 7" suggests 1:2:4 ratio roughly?
    # codes_0: 1 token
    # codes_1: 2 tokens
    # codes_2: 4 tokens
    # Total 7 tokens per "frame".
    
    dummy_c0 = torch.randint(0, 1024, (1, 10), dtype=torch.int32)
    dummy_c1 = torch.randint(0, 1024, (1, 20), dtype=torch.int32)
    dummy_c2 = torch.randint(0, 1024, (1, 40), dtype=torch.int32)
    
    print(f"Exporting to {output_path}...")
    torch.onnx.export(
        wrapper,
        (dummy_c0, dummy_c1, dummy_c2),
        output_path,
        input_names=['codes_0', 'codes_1', 'codes_2'],
        output_names=['audio'],
        dynamic_axes={
            'codes_0': {0: 'batch', 1: 'time'},
            'codes_1': {0: 'batch', 1: 'time'},
            'codes_2': {0: 'batch', 1: 'time'},
            'audio': {0: 'batch', 2: 'time'}
        },
        opset_version=18
    )
    print("Export complete.")

if __name__ == "__main__":
    export_model()
