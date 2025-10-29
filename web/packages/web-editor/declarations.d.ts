declare module "*.module.css";

declare module "*?raw" {
  const content: string;
  export default content;
}
