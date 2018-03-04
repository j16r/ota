// 
// Intro screen component
//
class IntroScreen extends React.Component {
  constructor(props) {
    super(props);
    this.state = {isOpen: false, mouseClickX: 0, mouseClickY: 0};
  }
  
  handleError(err) {
    console.error(err);
  }

  onClick = (event) => {
    this.setState({
      isOpen: !this.state.isOpen,
      mouseClickX: event.clientX,
      mouseClickY: event.clientY
    });
  }

  render() {
    return (
      <div onClick={this.onClick}>
        <section className="banner">
          <span>
            <div>
              <h1>
                ota
              </h1>
            </div>
            <div>
              <h2>
                click anywhere to create
              </h2>
            </div>
          </span>
        </section>
        <EditDialog show={this.state.isOpen} x={this.state.mouseClickX} y={this.state.mouseClickY}/>
      </div>
    )
  }
}

//
// Edit Dialog pops up a small text editor for updating or creating content.
//
class EditDialog extends React.Component {
  onSave = () => {
    console.log("onSave");
    this.setState({
      show: false,
    });
  }

  onCancel = () => {
    console.log("onCancel");
    this.setState({
      show: false,
    });
  }

  onClick = (event) => {
    event.stopPropagation();
  }

  render() {
    if(!this.props.show) {
      return null;
    }

    return (
      <span style={{"left": this.props.x, "top": this.props.y}} onClick={this.onClick} className="modal">
        <input type="textarea"/>
        <section className="footer">
          <button onClick={this.onSave}>
            Save
          </button>
          <button onClick={this.onCancel}>
            Close
          </button>
        </section>
      </span>
    );
  }
}

EditDialog.propTypes = {
  show: function() {return null},
  x: function() {return null},
  y: function() {return null},
};

const introScreen = <IntroScreen/>;
ReactDOM.render(
  introScreen,
  document.getElementById('root')
);
