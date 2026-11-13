import sys,json
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'engine' if (Path(__file__).resolve().parent.parent/'engine').is_dir() else Path(__file__).resolve().parent/'engine'))
from evidence import make_report,save
save(make_report(),sys.argv[1])
print('RELATORIO_CRIADO:',sys.argv[1])
