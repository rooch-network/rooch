export default function Example() {
  return (
    <div className="p-4">
      <button className="btn btn-primary">Click me</button>
      <p className="text-xl text-blue-500 mt-4">
        This should be blue and large
      </p>
      <details className="dropdown">
        <summary className="btn m-1">open or close</summary>
        <ul className="menu dropdown-content bg-base-100 rounded-box z-[1] w-52 p-2 shadow">
          <li>
            <a>Item 1</a>
          </li>
          <li>
            <a>Item 2</a>
          </li>
        </ul>
      </details>
    </div>
  );
}
