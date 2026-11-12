declare global{interface Window{lxo:{invoke:(channel:string,...args:any[])=>Promise<any>;onWeight:(callback:(w:any)=>void)=>()=>void}}}export {};
