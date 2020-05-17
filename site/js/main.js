//
// Intro screen component
//
class IntroScreen extends React.Component {
  constructor(props) {
    super(props);
    this.state = {isOpen: false, mouseClickX: 0, mouseClickY: 0, body: ''};
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
  constructor(props) {
    super(props);
    this.state = {
      show: props.show,
      inputValue: '',
      idValue: ''
    };
  }
  
  onSave = () => {
    console.log("onSave", this);

    fetch('/articles', {
      method: 'POST',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name: 'article',
        id: this.state.idValue,
        body: this.state.inputValue,
        properties: {},
        tags: [],
      })
    }).then((response) => {
      console.log("Got response: ", response, this);
      this.setState({
        show: false,
      });
    });
  }

  onCancel = () => {
    console.log("onCancel");
    this.setState({
      show: false,
    });
  }

  onClick(event) {
    event.stopPropagation();
  }

  updateInputValue(event) {
    this.setState({
      inputValue: event.target.value
    });
  }

  updateIdValue(event) {
    this.setState({
      idValue: event.target.value
    });
  }

  render() {
    if(!this.props.show) {
      return null;
    }

    return (
      <span style={{"left": this.props.x, "top": this.props.y}} onClick={this.onClick} className="modal">
        <div>
          <label>text</label>
          <input type="textarea" value={this.state.inputValue} onChange={event => this.updateInputValue(event)}/>
        </div>
        <div>
          <label>id</label>
          <input value={this.state.idValue} onChange={event => this.updateIdValue(event)}/>
        </div>
        <div>
          <section className="footer">
            <button onClick={this.onSave}>
              Save
            </button>
            <button onClick={this.onCancel}>
              Close
            </button>
          </section>
        </div>
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
