import torch
from pathlib import Path
import torch.nn as nn
import torch.fx as fx

class MyModule(nn.Module):
    def forward(self, a, b):
        x = torch.ops.aten.add.Tensor(a, b)
        y = torch.ops.aten.neg.default(a)
        return torch.ops.aten.add.Tensor(x, y)

m = MyModule()
gm = fx.symbolic_trace(m)
output_dir = Path(__file__).parent.parent / "data"
output_dir.mkdir(parents=True, exist_ok=True)
output_path = output_dir / "test_module.pt"
torch.save(gm, output_path)